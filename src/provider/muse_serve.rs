//! Muse Code through `muse serve`: the Muse Session Protocol (MSP v1, JSON-RPC 2.0,
//! one JSON object per line on stdio). Verified against Muse Code 1.4.14 on November 2:
//! a real shell approval arrives as `approval/request`, and `approval/decide` with
//! "Allow once" or "Reject" is recorded as `resolvedBy: user`.
//!
//! One `muse serve` process per run: initialize, start or resume the session, select the
//! approval mode, start the turn, then close when the turn completes.

use super::{EventSink, ProviderEvent, Run};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Ask the user for every action the host's rules do not already allow.
const APPROVAL_MODE: &str = "promptUnmatched";

/// What one incoming message means for the task.
#[derive(Debug, PartialEq)]
pub enum Incoming {
    /// The answer to one of our requests: `Ok(result)` or `Err(message)`.
    Response(u64, Result<Value, String>),
    /// A server request that needs a presentation receipt (`{}`), and maybe more.
    ServerRequest(Value, Box<Incoming>),
    Approval(Pending),
    Resolved {
        approval: String,
        summary: String,
    },
    Text(String),
    /// A finished tool call, and the files it changed (absolute paths).
    Step(String, Vec<String>),
    Tokens(String),
    TurnDone {
        ok: bool,
        summary: String,
    },
    Ignored,
}

/// A pending approval and the choices needed to answer it.
#[derive(Clone, Debug, PartialEq)]
pub struct Pending {
    /// The tool that asks (`bash`, `write_file`, `mcp__server__tool`, ...).
    pub tool: String,
    pub approval: String,
    pub session: String,
    pub requirement: Value,
    pub allow: Option<String>,
    pub deny: Option<String>,
    pub summary: String,
}

pub fn interpret(msg: &Value) -> Incoming {
    let method = msg["method"].as_str();
    if let (Some(id), None) = (msg["id"].as_u64(), method) {
        return Incoming::Response(
            id,
            match msg.get("error") {
                Some(error) => Err(error["message"]
                    .as_str()
                    .unwrap_or("unknown error")
                    .to_string()),
                None => Ok(msg["result"].clone()),
            },
        );
    }
    let p = &msg["params"];
    let inner = match method.unwrap_or_default() {
        "approval/request" | "approval/requested" => Incoming::Approval(pending(p)),
        "approval/resolved" => Incoming::Resolved {
            approval: p["approvalId"].as_str().unwrap_or_default().into(),
            summary: format!(
                "Muse recorded the decision: {} (by {}).",
                p["decision"].as_str().unwrap_or("unknown"),
                p["resolvedBy"].as_str().unwrap_or("unknown")
            ),
        },
        "item/completed" => item(&p["item"]),
        "session/tokenUsage" => {
            let c = &p["cumulative"];
            match c["totalTokens"].as_u64() {
                Some(total) => Incoming::Tokens(format!(
                    "{total} tokens ({} prompt, {} output)",
                    c["promptTokens"].as_u64().unwrap_or(0),
                    c["outputTokens"].as_u64().unwrap_or(0)
                )),
                None => Incoming::Ignored,
            }
        }
        "turn/completed" => {
            let terminal = p["terminal"].as_str().unwrap_or(if p["error"].is_null() {
                "unknown"
            } else {
                "failed"
            });
            let reason = p["reason"]
                .as_str()
                .or_else(|| p["error"]["message"].as_str())
                .or_else(|| p["error"].as_str());
            let ok = terminal == "completed";
            // Recorded November 2 with no credentials: error.kind "authRequired".
            let hint = if p["error"]["kind"].as_str() == Some("authRequired") {
                " Sign in to Muse: run `muse` in a terminal and type /login, then send a follow-up here to retry."
            } else {
                ""
            };
            Incoming::TurnDone {
                ok,
                summary: match reason {
                    Some(r) if !ok => {
                        format!("Muse turn {terminal}: {}.{hint}", r.trim_end_matches('.'))
                    }
                    _ if ok => "Muse finished the turn.".into(),
                    _ => format!("Muse turn {terminal}.{hint}"),
                },
            }
        }
        _ => Incoming::Ignored,
    };
    match msg["id"].clone() {
        Value::Null => inner,
        id => Incoming::ServerRequest(id, Box::new(inner)),
    }
}

fn pending(p: &Value) -> Pending {
    let choice = |decisions: &[&str]| {
        p["availableChoices"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["scope"].as_str() == Some("once"))
            .find(|c| decisions.contains(&c["decision"].as_str().unwrap_or_default()))
            .and_then(|c| c["choiceId"].as_str())
            .map(String::from)
    };
    let subject = &p["subject"];
    let what = match subject["kind"].as_str().unwrap_or_default() {
        "shell" => format!("run `{}`", subject["command"].as_str().unwrap_or("?")),
        "fileAccess" => format!(
            "{} {}",
            subject["access"].as_str().unwrap_or("access"),
            subject["path"].as_str().unwrap_or("?")
        ),
        "network" => format!("reach {}", subject["host"].as_str().unwrap_or("?")),
        _ => format!(
            "{} {}",
            p["toolName"].as_str().unwrap_or("tool"),
            short(p["rawArgs"].as_str().unwrap_or_default(), 120)
        ),
    };
    let reason = p["reviewerReason"]
        .as_str()
        .map(|r| format!(" — {r}"))
        .unwrap_or_default();
    Pending {
        approval: p["approvalId"].as_str().unwrap_or_default().into(),
        session: p["sessionId"].as_str().unwrap_or_default().into(),
        requirement: p["currentRequirementId"].clone(),
        tool: p["toolName"].as_str().unwrap_or_default().to_string(),
        allow: choice(&["approved"]),
        deny: choice(&["denied", "abort"]),
        summary: format!("{what}{reason}"),
    }
}

fn item(item: &Value) -> Incoming {
    match item["kind"].as_str().unwrap_or_default() {
        "agentMessage" => match item["text"].as_str() {
            Some(text) if !text.trim().is_empty() => Incoming::Text(text.trim().to_string()),
            _ => Incoming::Ignored,
        },
        "toolCall" => {
            let args: Value =
                serde_json::from_str(item["args"].as_str().unwrap_or("{}")).unwrap_or_default();
            let target = ["command", "path", "file_path", "pattern", "url"]
                .iter()
                .find_map(|k| args[*k].as_str())
                .map(|t| format!(" {}", short(t, 80)))
                .unwrap_or_default();
            let failure = item["failureReason"]
                .as_str()
                .map(|r| format!(" ({r})"))
                .unwrap_or_default();
            Incoming::Step(
                format!(
                    "{}{target}: {}{failure}",
                    item["tool"].as_str().unwrap_or("tool"),
                    item["status"].as_str().unwrap_or("done")
                ),
                changed_files(item, &args),
            )
        }
        _ => Incoming::Ignored,
    }
}

/// Files a completed tool call changed. Muse writes a patch record listing them
/// (`patchRef.path`); without one, fall back to the tool's `path` argument.
fn changed_files(item: &Value, args: &Value) -> Vec<String> {
    let changed = item["patchSummary"]["files"].as_u64().unwrap_or(0);
    if changed == 0 || item["status"].as_str() != Some("completed") {
        return Vec::new();
    }
    let from_patch = item["patchRef"]["path"]
        .as_str()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|patch| {
            patch["files"].as_array().map(|files| {
                files
                    .iter()
                    .filter_map(|f| f["path"].as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
        })
        .filter(|files| !files.is_empty());
    from_patch.unwrap_or_else(|| {
        args["path"]
            .as_str()
            .or(args["file_path"].as_str())
            .map(|p| vec![p.to_string()])
            .unwrap_or_default()
    })
}

fn short(text: &str, max: usize) -> String {
    let line = text.lines().next().unwrap_or_default();
    if line.chars().count() > max || text.lines().nth(1).is_some() {
        format!("{}…", line.chars().take(max).collect::<String>())
    } else {
        line.to_string()
    }
}

/// UUIDv7 for MSP `commandId`s: 48-bit Unix milliseconds, then random bits.
pub fn uuid7() -> String {
    let mut bytes = [0u8; 16];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = std::io::Read::read_exact(&mut f, &mut bytes);
    }
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    bytes[..6].copy_from_slice(&ms.to_be_bytes()[2..]);
    bytes[6] = 0x70 | (bytes[6] & 0x0f);
    bytes[8] = 0x80 | (bytes[8] & 0x3f);
    let h: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    )
}

#[derive(Default)]
struct State {
    next_id: u64,
    methods: HashMap<u64, &'static str>,
    session: Option<String>,
    turn: Option<String>,
    approvals: HashMap<String, Pending>,
    /// The user's choice per approval. Muse asks once per stage of a command
    /// (`a && b | c` gives one request per stage, same approval id); the user saw the
    /// whole command, so later stages get the same answer.
    decided: HashMap<String, bool>,
    /// (approval, requirement) pairs already handled: each request arrives twice.
    seen: std::collections::HashSet<(String, String)>,
    tokens: Option<String>,
    finished: bool,
    stderr: Vec<String>,
}

struct Shared {
    stdin: Mutex<Option<ChildStdin>>,
    state: Mutex<State>,
    child: Mutex<Child>,
    pid: u32,
    task: String,
    sink: EventSink,
}

impl Shared {
    fn emit(&self, event: ProviderEvent) {
        let _ = self.sink.send((self.task.clone(), event));
    }

    fn write(&self, msg: &Value) -> Result<(), String> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| "Muse connection lock poisoned")?;
        let stdin = stdin.as_mut().ok_or("The Muse connection is closed.")?;
        writeln!(stdin, "{msg}")
            .and_then(|()| stdin.flush())
            .map_err(|e| format!("Could not write to Muse: {e}"))
    }

    fn request(&self, method: &'static str, params: Value) -> Result<(), String> {
        let id = {
            let mut state = self.state.lock().map_err(|_| "Muse state lock poisoned")?;
            state.next_id += 1;
            let id = state.next_id;
            state.methods.insert(id, method);
            id
        };
        self.write(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
    }

    /// Report the outcome once, then close the connection so `muse serve` exits.
    fn finish(&self, ok: bool, summary: String) {
        let tokens = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if state.finished {
                return;
            }
            state.finished = true;
            state.tokens.clone()
        };
        let usage = match tokens {
            Some(t) => format!(" Usage this run: {t}."),
            None => " Muse reported no token usage for this run.".into(),
        };
        let end = if summary.ends_with(['.', '!', '?']) {
            ""
        } else {
            "."
        };
        self.emit(ProviderEvent::Finished {
            ok,
            summary: format!("{summary}{end}{usage}"),
        });
        if let Ok(mut stdin) = self.stdin.lock() {
            stdin.take();
        }
    }

    fn kill_group(&self) {
        // SAFETY: plain syscall; a negative pid signals the run's own process group.
        unsafe { libc::kill(-(self.pid as i32), libc::SIGTERM) };
    }
}

pub fn start(
    binary: &Path,
    task_id: &str,
    project_dir: &str,
    prompt: &str,
    session: Option<&str>,
    sink: EventSink,
) -> Result<Box<dyn Run>, String> {
    let mut child = Command::new(binary)
        .arg("serve")
        .current_dir(project_dir)
        // The built-in LLM judge would answer most requests itself; the user decides
        // instead (undocumented switch found in the 1.4.14 binary; D-010 proposal).
        .env("MUSE_DISABLE_APPROVAL_JUDGE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("Could not start {} serve: {e}", binary.display()))?;
    let pid = child.id();
    let stdout = child.stdout.take().ok_or("No stdout from muse serve")?;
    let stderr = child.stderr.take().ok_or("No stderr from muse serve")?;
    let shared = Arc::new(Shared {
        stdin: Mutex::new(child.stdin.take()),
        state: Mutex::new(State::default()),
        child: Mutex::new(child),
        pid,
        task: task_id.into(),
        sink,
    });
    {
        let shared = shared.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut state) = shared.state.lock() {
                    state.stderr.push(line);
                    if state.stderr.len() > 6 {
                        state.stderr.remove(0);
                    }
                }
            }
        });
    }
    shared.request(
        "initialize",
        json!({"clientInfo": {"name": "da_workstation", "title": "DA Workstation",
                              "version": env!("CARGO_PKG_VERSION")},
               // sessionMcp: the desktop tools below. No question dialogs in the
               // sidebar yet, so Muse must not wait on one.
               "capabilities": {"requestedCapabilities": ["sessionMcp"],
                                "userInputDialogs": false}}),
    )?;
    let plan = Plan {
        prompt: prompt.into(),
        resume: session.map(String::from),
        project_dir: project_dir.into(),
    };
    {
        let shared = shared.clone();
        std::thread::spawn(move || read_loop(shared, stdout, plan));
    }
    Ok(Box::new(ServeRun { shared }))
}

struct Plan {
    prompt: String,
    resume: Option<String>,
    project_dir: String,
}

fn read_loop(shared: Arc<Shared>, stdout: std::process::ChildStdout, plan: Plan) {
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Err(error) = handle(&shared, &plan, interpret(&msg)) {
            shared.finish(false, error);
        }
    }
    let tail = shared
        .state
        .lock()
        .map(|s| s.stderr.join(" | "))
        .unwrap_or_default();
    shared.finish(
        false,
        format!("Muse stopped before the turn finished. Last diagnostics: {tail}"),
    );
    let _ = shared.child.lock().map(|mut c| c.wait());
}

fn handle(shared: &Shared, plan: &Plan, incoming: Incoming) -> Result<(), String> {
    match incoming {
        Incoming::ServerRequest(id, inner) => {
            shared.write(&json!({"jsonrpc": "2.0", "id": id, "result": {}}))?;
            handle(shared, plan, *inner)
        }
        Incoming::Response(id, result) => {
            let method = shared
                .state
                .lock()
                .ok()
                .and_then(|mut s| s.methods.remove(&id))
                .unwrap_or("request");
            let result = result.map_err(|e| format!("Muse refused {method}: {e}"))?;
            match method {
                "initialize" => {
                    shared
                        .write(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}))?;
                    match &plan.resume {
                        Some(session) => shared.request(
                            "session/resume",
                            json!({"commandId": uuid7(), "sessionId": session,
                                   "config": workstation_tools()}),
                        ),
                        None => shared.request(
                            "session/start",
                            json!({"commandId": uuid7(), "approvalMode": APPROVAL_MODE,
                                   "workspaceRoot": plan.project_dir,
                                   "config": workstation_tools()}),
                        ),
                    }
                }
                "session/start" | "session/resume" => {
                    let session = result["session"]["sessionId"]
                        .as_str()
                        .or(plan.resume.as_deref())
                        .ok_or("Muse did not return a session id")?
                        .to_string();
                    if let Ok(mut state) = shared.state.lock() {
                        state.session = Some(session.clone());
                    }
                    shared.emit(ProviderEvent::Started {
                        session_ref: session.clone(),
                    });
                    shared.emit(ProviderEvent::Diagnostic(format!(
                        "Open this session in a terminal: cd {} && muse resume {session}",
                        plan.project_dir
                    )));
                    if method == "session/resume" {
                        shared.request(
                            "session/setApprovalMode",
                            json!({"commandId": uuid7(), "sessionId": session,
                                   "mode": APPROVAL_MODE}),
                        )?;
                    }
                    shared.request(
                        "turn/start",
                        json!({"commandId": uuid7(), "sessionId": session,
                               "input": [{"type": "text", "text": plan.prompt}]}),
                    )
                }
                "turn/start" => {
                    if let Ok(mut state) = shared.state.lock() {
                        state.turn = result["turnId"].as_str().map(String::from);
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        Incoming::Approval(pending) => {
            enum Next {
                Ask,
                Repeat(bool),
                Nothing,
            }
            let next = shared
                .state
                .lock()
                .map(|mut s| {
                    let first = s
                        .approvals
                        .insert(pending.approval.clone(), pending.clone())
                        .is_none();
                    let key = (pending.approval.clone(), pending.requirement.to_string());
                    if !s.seen.insert(key) {
                        return Next::Nothing;
                    }
                    match s.decided.get(&pending.approval) {
                        Some(&allow) => Next::Repeat(allow),
                        None if first => Next::Ask,
                        None => Next::Nothing,
                    }
                })
                .unwrap_or(Next::Nothing);
            // The workstation's own desktop tools run without asking the user.
            let next = match next {
                Next::Ask if crate::mcp::is_own_tool(&pending.tool) => {
                    if let Ok(mut s) = shared.state.lock() {
                        s.decided.insert(pending.approval.clone(), true);
                    }
                    Next::Repeat(true)
                }
                other => other,
            };
            match next {
                Next::Ask => shared.emit(ProviderEvent::NeedsApproval {
                    id: pending.approval,
                    summary: pending.summary,
                }),
                Next::Repeat(allow) => {
                    if let Err(error) = decide(shared, &pending, allow) {
                        shared.emit(ProviderEvent::Diagnostic(format!(
                            "Could not answer the next step of the request: {error}"
                        )));
                    }
                }
                Next::Nothing => {}
            }
            Ok(())
        }
        Incoming::Resolved { approval, summary } => {
            if let Ok(mut state) = shared.state.lock() {
                state.approvals.remove(&approval);
            }
            shared.emit(ProviderEvent::ApprovalResolved {
                id: approval,
                summary,
            });
            Ok(())
        }
        Incoming::Text(text) => {
            shared.emit(ProviderEvent::Output(text));
            Ok(())
        }
        Incoming::Step(step, files) => {
            shared.emit(ProviderEvent::Diagnostic(step));
            if !files.is_empty() {
                shared.emit(ProviderEvent::FilesChanged(files));
            }
            Ok(())
        }
        Incoming::Tokens(tokens) => {
            if let Ok(mut state) = shared.state.lock() {
                state.tokens = Some(tokens);
            }
            Ok(())
        }
        Incoming::TurnDone { ok, summary } => {
            shared.finish(ok, summary);
            Ok(())
        }
        Incoming::Ignored => Ok(()),
    }
}

/// The workstation's desktop tools, offered to every Muse session as an MCP server.
fn workstation_tools() -> Value {
    // DA_WORKSTATION_BIN: tests run inside a test binary, not the app.
    let exe = match std::env::var_os("DA_WORKSTATION_BIN") {
        Some(path) => std::path::PathBuf::from(path),
        None => match std::env::current_exe() {
            Ok(exe) => exe,
            Err(_) => return json!({}),
        },
    };
    // Muse starts tool servers with a bare environment; the tools need the desktop's.
    let env: serde_json::Map<String, Value> = [
        "DBUS_SESSION_BUS_ADDRESS",
        "WAYLAND_DISPLAY",
        "DISPLAY",
        "XDG_RUNTIME_DIR",
        "XDG_DATA_DIRS",
        "XDG_DATA_HOME",
        "XDG_CURRENT_DESKTOP",
        "HOME",
        "PATH",
        "LANG",
    ]
    .into_iter()
    .filter_map(|key| {
        std::env::var(key)
            .ok()
            .map(|v| (key.to_string(), Value::from(v)))
    })
    .collect();
    json!({"mcpServers": {crate::mcp::SERVER: {
        "transport": "stdio",
        "command": exe.to_string_lossy(),
        "args": ["--mcp"],
        "env": env,
        "framing": "lineDelimitedJson",
        // A tool server that fails to start must not stop the task.
        "mode": "optional"
    }}})
}

/// Send the user's choice for the request's current requirement (one stage).
fn decide(shared: &Shared, pending: &Pending, allow: bool) -> Result<(), String> {
    let choice = if allow {
        pending.allow.clone()
    } else {
        pending.deny.clone()
    }
    .ok_or("Muse offered no matching one-time choice for this request.")?;
    shared.request(
        "approval/decide",
        json!({"commandId": uuid7(), "sessionId": pending.session,
               "approvalId": pending.approval, "choiceId": choice,
               "requirementId": pending.requirement}),
    )
}

struct ServeRun {
    shared: Arc<Shared>,
}

impl Run for ServeRun {
    fn answer_approval(&mut self, id: &str, allow: bool) -> Result<(), String> {
        let pending = {
            let mut state = self
                .shared
                .state
                .lock()
                .map_err(|_| "Muse state lock poisoned")?;
            let pending = state
                .approvals
                .get(id)
                .cloned()
                .ok_or("Muse has already resolved this request.")?;
            // Set before sending: Muse may ask for the next stage right after.
            state.decided.insert(id.to_string(), allow);
            pending
        };
        decide(&self.shared, &pending, allow)
    }

    fn cancel(&mut self) -> Result<(), String> {
        let (session, turn) = {
            let state = self
                .shared
                .state
                .lock()
                .map_err(|_| "Muse state lock poisoned")?;
            if state.finished {
                return Ok(());
            }
            (state.session.clone(), state.turn.clone())
        };
        let asked = match session {
            Some(session) => self
                .shared
                .request(
                    "turn/interrupt",
                    json!({"commandId": uuid7(), "sessionId": session, "turnId": turn}),
                )
                .is_ok(),
            None => false,
        };
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            // Give Muse a moment to close the turn itself; then end the process group.
            if asked {
                std::thread::sleep(Duration::from_secs(5));
            }
            let finished = shared.state.lock().map(|s| s.finished).unwrap_or(true);
            if !finished {
                shared.kill_group();
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shapes recorded from Muse Code 1.4.14 `muse serve` on November 2 (trimmed).
    const APPROVAL: &str = r#"{"id":1,"jsonrpc":"2.0","method":"approval/request","params":{"approvalId":"a1","availableChoices":[{"choiceId":"allow_once","decision":"approved","label":"Allow once","scope":"once"},{"acceptsFeedback":true,"choiceId":"abort","decision":"abort","label":"Reject","scope":"once"}],"currentRequirementId":{"approvalId":"a1","sourceIndex":0},"rawArgs":"{\"command\":\"date > probe.txt\"}","sessionId":"s1","subject":{"command":"date > probe.txt","kind":"shell"},"toolName":"bash"}}"#;

    #[test]
    fn approval_request_needs_a_receipt_and_offers_allow_and_reject() {
        let msg: Value = serde_json::from_str(APPROVAL).unwrap();
        let Incoming::ServerRequest(id, inner) = interpret(&msg) else {
            panic!()
        };
        assert_eq!(id, json!(1));
        let Incoming::Approval(p) = *inner else {
            panic!()
        };
        assert_eq!(p.summary, "run `date > probe.txt`");
        assert_eq!(p.allow.as_deref(), Some("allow_once"));
        assert_eq!(p.deny.as_deref(), Some("abort"));
        assert_eq!(p.requirement, json!({"approvalId": "a1", "sourceIndex": 0}));
    }

    #[test]
    fn file_edits_report_the_changed_files() {
        let dir = std::env::temp_dir().join(format!("da-patch-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let patch = dir.join("tool_patch.json");
        // Patch record shape from a real `edit_file` call (November 2).
        std::fs::write(
            &patch,
            r#"{"files":[{"hunks":[{"lines":["-hello","+hi"]}],"path":"/work/notes.txt"}]}"#,
        )
        .unwrap();
        let item = json!({"kind": "toolCall", "tool": "edit_file", "status": "completed",
            "args": "{\"find\":\"hello\",\"path\":\"notes.txt\",\"replace\":\"hi\"}",
            "patchRef": {"path": patch.to_str().unwrap()},
            "patchSummary": {"added": 1, "files": 1, "removed": 1}});
        let msg = json!({"method": "item/completed", "params": {"item": item}});
        assert_eq!(
            interpret(&msg),
            Incoming::Step(
                "edit_file notes.txt: completed".into(),
                vec!["/work/notes.txt".into()]
            )
        );
        // Without a readable patch record, the tool's own path argument is used.
        let mut item = item;
        item["patchRef"]["path"] = json!("/nonexistent/patch.json");
        let msg = json!({"method": "item/completed", "params": {"item": item}});
        let Incoming::Step(_, files) = interpret(&msg) else {
            panic!()
        };
        assert_eq!(files, vec!["notes.txt".to_string()]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn items_turns_and_errors_are_read() {
        let v = |s: &str| serde_json::from_str::<Value>(s).unwrap();
        assert_eq!(
            interpret(&v(
                r#"{"method":"item/completed","params":{"item":{"kind":"agentMessage","text":"Ran it.\n"}}}"#
            )),
            Incoming::Text("Ran it.".into())
        );
        assert_eq!(
            interpret(&v(
                r#"{"method":"item/completed","params":{"item":{"kind":"toolCall","tool":"bash","args":"{\"command\":\"date > probe.txt\"}","status":"rejected","failureReason":"approval aborted"}}}"#
            )),
            Incoming::Step(
                "bash date > probe.txt: rejected (approval aborted)".into(),
                Vec::new()
            )
        );
        assert_eq!(
            interpret(&v(
                r#"{"method":"approval/resolved","params":{"approvalId":"a1","decision":"abort","resolvedBy":"user"}}"#
            )),
            Incoming::Resolved {
                approval: "a1".into(),
                summary: "Muse recorded the decision: abort (by user).".into()
            }
        );
        assert_eq!(
            interpret(&v(
                r#"{"method":"turn/completed","params":{"terminal":"completed"}}"#
            )),
            Incoming::TurnDone {
                ok: true,
                summary: "Muse finished the turn.".into()
            }
        );
        assert_eq!(
            interpret(&v(
                r#"{"method":"turn/completed","params":{"terminal":"failed","reason":"API error 402"}}"#
            )),
            Incoming::TurnDone {
                ok: false,
                summary: "Muse turn failed: API error 402.".into()
            }
        );
        // Recorded with no Muse credentials (trimmed); no `terminal` field.
        let Incoming::TurnDone { ok, summary } = interpret(&v(
            r#"{"method":"turn/completed","params":{"durationMs":6,"error":{"kind":"authRequired","message":"not logged in: run /login to add an API key","retryable":false},"reason":"not logged in: run /login to add an API key"}}"#,
        )) else {
            panic!()
        };
        assert!(!ok);
        assert!(
            summary.starts_with("Muse turn failed: not logged in"),
            "{summary}"
        );
        assert!(
            summary.contains("run `muse` in a terminal and type /login"),
            "{summary}"
        );
        assert_eq!(
            interpret(&v(
                r#"{"error":{"code":-32600,"message":"Not initialized"},"id":2,"jsonrpc":"2.0"}"#
            )),
            Incoming::Response(2, Err("Not initialized".into()))
        );
        assert_eq!(
            interpret(&v(
                r#"{"method":"session/tokenUsage","params":{"cumulative":{"outputTokens":257,"promptTokens":28326,"totalTokens":28583}}}"#
            )),
            Incoming::Tokens("28583 tokens (28326 prompt, 257 output)".into())
        );
    }

    #[test]
    fn command_ids_are_uuid_v7() {
        let id = uuid7();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "7");
        assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"));
        assert_ne!(id, uuid7());
    }
}
