//! Muse Code adapter. Default transport: `muse serve` (see `muse_serve`), which carries
//! approvals. Fallback (`DA_MUSE_TRANSPORT=exec`): `muse exec --json --session-id <uuid>`
//! in the project folder, reading Muse's JSONL events; it cannot answer approvals.
//! Reusing a session id keeps the model's context (verified November 2).

use super::{EventSink, Provider, ProviderEvent, Run};
use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub const INSTALL_HINT: &str = "Muse Code is not installed. Install it with: curl -fsSL https://dev.meta.ai/install.sh | sh — then run `muse` and sign in with /login.";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Transport {
    Serve,
    Exec,
}

pub struct Muse {
    /// Tests point this at a stand-in script.
    binary: Option<PathBuf>,
    transport: Transport,
}

impl Default for Muse {
    fn default() -> Self {
        let exec = std::env::var("DA_MUSE_TRANSPORT").is_ok_and(|t| t == "exec");
        Self {
            binary: None,
            transport: if exec {
                Transport::Exec
            } else {
                Transport::Serve
            },
        }
    }
}

/// Find the real `muse` binary. A shell function of the same name does not count.
pub fn find_binary(path_var: Option<&str>, home: Option<&Path>) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = path_var
        .unwrap_or_default()
        .split(':')
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .collect();
    if let Some(home) = home {
        for extra in [".local/bin", ".muse/bin", ".meta/bin"] {
            dirs.push(home.join(extra));
        }
    }
    dirs.into_iter().map(|d| d.join("muse")).find(|p| {
        std::fs::metadata(p).is_ok_and(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.is_file() && m.permissions().mode() & 0o111 != 0
        })
    })
}

fn binary() -> Option<PathBuf> {
    let path = std::env::var("PATH").ok();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    find_binary(path.as_deref(), home.as_deref())
}

impl Muse {
    fn binary(&self) -> Option<PathBuf> {
        self.binary.clone().or_else(binary)
    }
}

impl Provider for Muse {
    fn id(&self) -> &'static str {
        "muse"
    }

    fn name(&self) -> &'static str {
        "Muse"
    }

    fn unavailable_reason(&self) -> Option<String> {
        self.binary().is_none().then(|| INSTALL_HINT.to_string())
    }

    fn resumes(&self, session_ref: &str) -> bool {
        is_uuid(session_ref)
    }

    fn start(
        &self,
        task_id: &str,
        project_dir: &str,
        prompt: &str,
        session: Option<&str>,
        sink: EventSink,
    ) -> Result<Box<dyn Run>, String> {
        let binary = self.binary().ok_or(INSTALL_HINT)?;
        if !Path::new(project_dir).is_dir() {
            return Err(format!("Project folder does not exist: {project_dir}"));
        }
        if self.transport == Transport::Serve {
            let session = session.filter(|s| is_uuid(s));
            return super::muse_serve::start(&binary, task_id, project_dir, prompt, session, sink);
        }
        let session = match session.filter(|s| is_uuid(s)) {
            Some(session) => session.to_string(),
            None => new_session_id()?,
        };
        let mut child = Command::new(&binary)
            .args(["exec", "--json", "--session-id", &session])
            .arg(prompt)
            .current_dir(project_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Own process group, so Stop ends Muse and everything it started.
            .process_group(0)
            .spawn()
            .map_err(|e| format!("Could not start {}: {e}", binary.display()))?;
        let pid = child.id();
        let _ = sink.send((
            task_id.into(),
            ProviderEvent::Started {
                session_ref: session.clone(),
            },
        ));
        let _ = sink.send((
            task_id.into(),
            ProviderEvent::Diagnostic(format!(
                "Open this session in a terminal: cd {project_dir} && muse resume {session}"
            )),
        ));
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));
        let terminal = Arc::new(Mutex::new(None));
        let readers = [
            stdout.map(|s| events(s, task_id, sink.clone(), terminal.clone())),
            stderr.map(|s| pipe(s, task_id, sink.clone(), ProviderEvent::Diagnostic)),
        ];
        let waiter = child.clone();
        let id = task_id.to_string();
        std::thread::spawn(move || {
            for reader in readers.into_iter().flatten() {
                let _ = reader.join();
            }
            let status = loop {
                match waiter.lock().map(|mut c| c.try_wait()) {
                    Ok(Ok(Some(status))) => break Ok(status),
                    Ok(Ok(None)) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Ok(Err(e)) => break Err(e.to_string()),
                    Err(_) => break Err("process lock poisoned".into()),
                }
            };
            let terminal = terminal.lock().ok().and_then(|mut t| t.take());
            let (ok, summary) = match (terminal, status) {
                (Some((ok, summary)), _) => (ok, summary),
                (None, Ok(s)) if s.success() => (true, "Muse exited with status 0.".to_string()),
                (None, Ok(s)) => (false, format!("Muse exited: {s}.")),
                (None, Err(e)) => (false, format!("Lost the Muse process: {e}")),
            };
            let _ = sink.send((id, ProviderEvent::Finished { ok, summary }));
        });
        Ok(Box::new(ExecRun { child, pid }))
    }
}

fn is_uuid(text: &str) -> bool {
    text.len() == 36
        && text.char_indices().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

fn new_session_id() -> Result<String, String> {
    std::fs::read_to_string("/proc/sys/kernel/random/uuid")
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Could not make a Muse session id: {e}"))
}

/// One line of `muse exec --json` output, reduced to what the task needs.
#[derive(Debug, PartialEq)]
enum Line {
    /// A piece of the answer; pieces are not split at line ends.
    Delta(String),
    /// A finished tool call, e.g. "write_file hello.txt: success".
    Tool(String),
    /// The run ended: completed or not, and the reason Muse gives.
    Terminal {
        ok: bool,
        summary: String,
    },
    Ignored,
    /// Not JSON: shown as it is.
    Text(String),
}

fn parse_line(line: &str) -> Line {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return Line::Text(line.to_string());
    };
    let kind = value["payload_type"].as_str().unwrap_or_default();
    let payload = &value["payload"];
    if kind == "run.output.delta" {
        return Line::Delta(payload["text"].as_str().unwrap_or_default().to_string());
    }
    if kind == "tool.result" {
        let facts = &payload["correlation_facts"];
        let name = facts["tool_name"].as_str().unwrap_or("tool");
        let outcome = facts["outcome"].as_str().unwrap_or("done");
        return Line::Tool(match payload["edit_facts"]["path"].as_str() {
            Some(path) => format!("{name} {path}: {outcome}"),
            None => format!("{name}: {outcome}"),
        });
    }
    if let Some(terminal) = kind.strip_prefix("run.terminal.") {
        let reason = payload["reason"].as_str().filter(|r| !r.is_empty());
        let ok = terminal == "completed";
        let summary = match (ok, reason) {
            (true, _) => "Muse finished the run.".to_string(),
            (false, Some(reason)) => format!("Muse run {terminal}: {reason}"),
            (false, None) => format!("Muse run {terminal}."),
        };
        return Line::Terminal { ok, summary };
    }
    Line::Ignored
}

type Terminal = Arc<Mutex<Option<(bool, String)>>>;

/// Read Muse's JSONL events: answer text goes out line by line; the outcome is kept.
fn events(
    stream: impl Read + Send + 'static,
    task_id: &str,
    sink: EventSink,
    terminal: Terminal,
) -> std::thread::JoinHandle<()> {
    let id = task_id.to_string();
    std::thread::spawn(move || {
        let mut text = String::new();
        let send = |line: String| sink.send((id.clone(), ProviderEvent::Output(line))).is_ok();
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            match parse_line(&line) {
                Line::Delta(delta) => {
                    text.push_str(&delta);
                    while let Some(end) = text.find('\n') {
                        let rest = text.split_off(end + 1);
                        text.pop();
                        if !send(std::mem::replace(&mut text, rest)) {
                            return;
                        }
                    }
                }
                Line::Tool(step) => {
                    if sink
                        .send((id.clone(), ProviderEvent::Diagnostic(step)))
                        .is_err()
                    {
                        return;
                    }
                }
                Line::Terminal { ok, summary } => {
                    if let Ok(mut t) = terminal.lock() {
                        *t = Some((ok, summary));
                    }
                }
                Line::Text(line) => {
                    if !send(line) {
                        return;
                    }
                }
                Line::Ignored => {}
            }
        }
        if !text.is_empty() {
            send(text);
        }
    })
}

fn pipe(
    stream: impl Read + Send + 'static,
    task_id: &str,
    sink: EventSink,
    wrap: fn(String) -> ProviderEvent,
) -> std::thread::JoinHandle<()> {
    let id = task_id.to_string();
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if sink.send((id.clone(), wrap(line))).is_err() {
                break;
            }
        }
    })
}

struct ExecRun {
    child: Arc<Mutex<Child>>,
    pid: u32,
}

impl Run for ExecRun {
    fn answer_approval(&mut self, _id: &str, _allow: bool) -> Result<(), String> {
        Err("This Muse connection (muse exec) cannot answer approval requests yet.".into())
    }

    fn cancel(&mut self) -> Result<(), String> {
        let running = self
            .child
            .lock()
            .map(|mut c| matches!(c.try_wait(), Ok(None)))
            .unwrap_or(false);
        if !running {
            return Ok(());
        }
        // SAFETY: plain syscall; a negative pid signals the run's own process group.
        if unsafe { libc::kill(-(self.pid as i32), libc::SIGTERM) } != 0 {
            return Err(format!(
                "Could not stop Muse: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_events_from_a_real_muse_run_are_read() {
        // Shapes recorded from Muse Code 1.4.14 (`muse exec --json`).
        let delta = r#"{"schema_version":1,"payload_type":"run.output.delta","payload":{"kind":"run_output_delta","text":"echo: first"}}"#;
        let done = r#"{"payload_type":"run.terminal.completed","payload":{"kind":"run_terminal","reason":null,"terminal":"completed","text":"echo: first"}}"#;
        let failed = r#"{"payload_type":"run.terminal.failed","payload":{"reason":"API error 402: Billing verification failed.","terminal":"failed","text":""}}"#;
        let other = r#"{"payload_type":"task.lifecycle.started","payload":{}}"#;
        assert_eq!(parse_line(delta), Line::Delta("echo: first".into()));
        assert_eq!(
            parse_line(done),
            Line::Terminal {
                ok: true,
                summary: "Muse finished the run.".into()
            }
        );
        assert_eq!(
            parse_line(failed),
            Line::Terminal {
                ok: false,
                summary: "Muse run failed: API error 402: Billing verification failed.".into()
            }
        );
        assert_eq!(parse_line(other), Line::Ignored);
        let tool = r#"{"payload_type":"tool.result","payload":{"correlation_facts":{"outcome":"success","tool_name":"write_file"},"edit_facts":{"added":1,"path":"hello.txt"}}}"#;
        assert_eq!(
            parse_line(tool),
            Line::Tool("write_file hello.txt: success".into())
        );
        assert!(is_uuid("4d2a50f3-508b-45e2-a72c-696b1a2da6b9"));
        assert!(!is_uuid("muse exec pid 42"));
        assert_eq!(parse_line("plain"), Line::Text("plain".into()));
    }

    #[test]
    fn shell_function_names_do_not_count_as_installed() {
        let empty = std::env::temp_dir().join(format!("da-muse-empty-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(find_binary(Some(empty.to_str().unwrap()), None), None);
    }

    #[test]
    fn executable_on_path_is_found() {
        let dir = std::env::temp_dir().join(format!("da-muse-bin-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("muse");
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(find_binary(Some(dir.to_str().unwrap()), None), Some(bin));
    }

    /// Held from writing a stand-in script until its process starts: a script being
    /// written while another test forks can fail to run with "Text file busy".
    static SPAWN: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn stand_in(script: &str) -> (Muse, PathBuf, std::sync::MutexGuard<'static, ()>) {
        use std::os::unix::fs::PermissionsExt;
        let guard = SPAWN.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "da-muse-run-{}-{}",
            std::process::id(),
            script.len()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("muse");
        std::fs::write(&bin, script).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        (
            Muse {
                binary: Some(bin),
                transport: Transport::Exec,
            },
            dir,
            guard,
        )
    }

    fn collect(rx: &std::sync::mpsc::Receiver<(String, ProviderEvent)>) -> Vec<ProviderEvent> {
        let mut events = Vec::new();
        while let Ok((_, event)) = rx.recv_timeout(std::time::Duration::from_secs(5)) {
            let done = matches!(event, ProviderEvent::Finished { .. });
            events.push(event);
            if done {
                break;
            }
        }
        events
    }

    #[test]
    fn exec_streams_output_in_the_project_folder_and_reports_exit() {
        let (muse, dir, _spawn) =
            stand_in("#!/bin/sh\necho \"$1 $2 $3 $5\"\npwd\necho oops >&2\nexit 3\n");
        let (tx, rx) = std::sync::mpsc::channel();
        let _run = muse
            .start("t1", dir.to_str().unwrap(), "hello", None, tx)
            .unwrap();
        let events = collect(&rx);
        assert!(
            matches!(events[0], ProviderEvent::Started { .. }),
            "{events:?}"
        );
        assert!(
            events.contains(&ProviderEvent::Output(
                "exec --json --session-id hello".into()
            )),
            "{events:?}"
        );
        assert!(
            events.contains(&ProviderEvent::Output(dir.display().to_string())),
            "{events:?}"
        );
        assert!(
            events.contains(&ProviderEvent::Diagnostic("oops".into())),
            "{events:?}"
        );
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: false, .. })
            ),
            "{events:?}"
        );
    }

    #[test]
    fn deltas_become_whole_lines_and_the_terminal_event_decides_the_outcome() {
        let d = |t: &str| {
            format!(r#"{{"payload_type":"run.output.delta","payload":{{"text":"{t}"}}}}"#)
        };
        let script = format!(
            "#!/bin/sh\ncat <<'EOF'\n{}\n{}\n{}\n{}\nEOF\nexit 0\n",
            d("Hel"),
            d(r"lo\nwor"),
            d("ld"),
            r#"{"payload_type":"run.terminal.failed","payload":{"reason":"Billing issue","terminal":"failed"}}"#
        );
        let (muse, dir, _spawn) = stand_in(&script);
        let (tx, rx) = std::sync::mpsc::channel();
        let _run = muse
            .start("t4", dir.to_str().unwrap(), "x", None, tx)
            .unwrap();
        let events = collect(&rx);
        assert!(
            events.contains(&ProviderEvent::Output("Hello".into())),
            "{events:?}"
        );
        assert!(
            events.contains(&ProviderEvent::Output("world".into())),
            "{events:?}"
        );
        assert!(
            matches!(events.last(), Some(ProviderEvent::Finished { ok: false, summary }) if summary.contains("Billing issue")),
            "{events:?}"
        );
    }

    /// Needs Muse Code installed; uses its offline echo provider. `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn real_muse_echo_run_end_to_end() {
        let real = binary().expect("muse installed");
        let script = format!(
            "#!/bin/sh\nshift\nexec '{}' exec --provider echo \"$@\"\n",
            real.display()
        );
        let (muse, dir, _spawn) = stand_in(&script);
        let (tx, rx) = std::sync::mpsc::channel();
        let _run = muse
            .start("t5", dir.to_str().unwrap(), "ping one", None, tx)
            .unwrap();
        let events = collect(&rx);
        assert!(
            events.contains(&ProviderEvent::Output("echo: ping one".into())),
            "{events:?}"
        );
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: true, .. })
            ),
            "{events:?}"
        );
    }

    /// Needs Muse Code installed and signed in; spends two small real runs.
    /// `cargo test -- --ignored real_muse_follow_up`
    #[test]
    #[ignore]
    fn real_muse_follow_up_keeps_context() {
        let muse = Muse::default();
        let dir = std::env::temp_dir().join(format!("da-muse-real-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dir = dir.to_str().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let _run = muse
            .start(
                "r1",
                dir,
                "Remember the code word HERON. Reply only OK.",
                None,
                tx.clone(),
            )
            .unwrap();
        let first = collect_slow(&rx);
        let Some(ProviderEvent::Started { session_ref }) = first.first().cloned() else {
            panic!("{first:?}");
        };
        assert!(muse.resumes(&session_ref));
        assert!(
            matches!(first.last(), Some(ProviderEvent::Finished { ok: true, .. })),
            "{first:?}"
        );
        let _run = muse
            .start(
                "r1",
                dir,
                "What code word did I give you? Reply with just the word.",
                Some(&session_ref),
                tx,
            )
            .unwrap();
        let second = collect_slow(&rx);
        assert!(
            second
                .iter()
                .any(|e| matches!(e, ProviderEvent::Output(t) if t.contains("HERON"))),
            "{second:?}"
        );
    }

    /// Needs Muse Code installed and signed in; spends small real runs.
    /// `cargo test -- --ignored real_muse_serve_approvals`
    #[test]
    #[ignore]
    fn real_muse_serve_approvals_reject_then_allow() {
        let muse = Muse::default();
        assert_eq!(muse.transport, Transport::Serve);
        let dir = std::env::temp_dir().join(format!("da-muse-approve-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let probe = dir.join("probe.txt");
        let prompt =
            "Use the shell tool to run exactly: date > probe.txt . Then say what happened.";
        for allow in [false, true] {
            let _ = std::fs::remove_file(&probe);
            let (tx, rx) = std::sync::mpsc::channel();
            let mut run = muse
                .start("a1", dir.to_str().unwrap(), prompt, None, tx)
                .unwrap();
            let mut events = Vec::new();
            while let Ok((_, event)) = rx.recv_timeout(std::time::Duration::from_secs(180)) {
                if let ProviderEvent::NeedsApproval { id, .. } = &event {
                    run.answer_approval(id, allow).unwrap();
                }
                let done = matches!(event, ProviderEvent::Finished { .. });
                events.push(event);
                if done {
                    break;
                }
            }
            assert!(
                events.iter().any(|e| matches!(e, ProviderEvent::NeedsApproval { summary, .. } if summary.contains("date > probe.txt"))),
                "{events:?}"
            );
            assert!(
                events.iter().any(|e| matches!(e, ProviderEvent::ApprovalResolved { summary, .. } if summary.contains("by user"))),
                "{events:?}"
            );
            assert!(
                matches!(
                    events.last(),
                    Some(ProviderEvent::Finished { ok: true, .. })
                ),
                "{events:?}"
            );
            assert_eq!(probe.exists(), allow, "allow={allow} {events:?}");
            eprintln!("allow={allow}: {:?}", events.last());
        }
    }

    /// A command with several stages (`a && b | c`): Muse asks once per stage with the
    /// same approval id. One user answer must cover the whole command.
    /// Needs Muse Code installed and signed in. `cargo test -- --ignored real_muse_multi`
    #[test]
    #[ignore]
    fn real_muse_multi_stage_command_needs_one_answer() {
        let muse = Muse::default();
        let dir = std::env::temp_dir().join(format!("da-muse-multi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("out.txt");
        let _ = std::fs::remove_file(&out);
        let prompt = "Use the shell tool to run exactly this one command and nothing else: \
                      ls -la && echo staged > out.txt && date | head -n 1";
        let (tx, rx) = std::sync::mpsc::channel();
        let mut run = muse
            .start("m1", dir.to_str().unwrap(), prompt, None, tx)
            .unwrap();
        let mut events = Vec::new();
        let mut asked = 0;
        while let Ok((_, event)) = rx.recv_timeout(std::time::Duration::from_secs(180)) {
            if let ProviderEvent::NeedsApproval { id, .. } = &event {
                asked += 1;
                run.answer_approval(id, true).unwrap();
            }
            let done = matches!(event, ProviderEvent::Finished { .. });
            events.push(event);
            if done {
                break;
            }
        }
        assert_eq!(asked, 1, "one question for the whole command: {events:?}");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ProviderEvent::ApprovalResolved { .. })),
            "{events:?}"
        );
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: true, .. })
            ),
            "{events:?}"
        );
        assert!(out.exists(), "{events:?}");
    }

    /// Muse uses the workstation's own tools without asking the user.
    /// Needs Muse Code, a signed-in account and the GNOME extension. `cargo test -- --ignored real_muse_tools`
    #[test]
    #[ignore]
    fn real_muse_tools_run_without_asking() {
        // SAFETY: set before any thread of this test reads it.
        unsafe {
            std::env::set_var(
                "DA_WORKSTATION_BIN",
                concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/da-workstation"),
            )
        };
        let muse = Muse::default();
        let dir = std::env::temp_dir().join(format!("da-muse-tools-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let prompt = "Using the da_workstation tools (not the shell), list my open windows \
                      and tell me how many there are and which app is focused.";
        let (tx, rx) = std::sync::mpsc::channel();
        let _run = muse
            .start("w1", dir.to_str().unwrap(), prompt, None, tx)
            .unwrap();
        let mut events = Vec::new();
        while let Ok((_, event)) = rx.recv_timeout(std::time::Duration::from_secs(180)) {
            let done = matches!(event, ProviderEvent::Finished { .. });
            events.push(event);
            if done {
                break;
            }
        }
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, ProviderEvent::NeedsApproval { .. })),
            "the user was asked: {events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(e, ProviderEvent::Diagnostic(d) if d.contains("list_windows") && d.contains("completed"))),
            "{events:?}"
        );
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: true, .. })
            ),
            "{events:?}"
        );
        eprintln!(
            "{:?}",
            events
                .iter()
                .filter(|e| matches!(e, ProviderEvent::Output(_)))
                .collect::<Vec<_>>()
        );
    }

    /// Needs Muse Code installed and signed in. `cargo test -- --ignored real_muse_stop`
    #[test]
    #[ignore]
    fn real_muse_stop_ends_the_turn() {
        let muse = Muse::default();
        let dir = std::env::temp_dir().join(format!("da-muse-stop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut run = muse
            .start(
                "s1",
                dir.to_str().unwrap(),
                "Write a 3000-word essay about rivers.",
                None,
                tx,
            )
            .unwrap();
        while !matches!(
            rx.recv_timeout(std::time::Duration::from_secs(60))
                .unwrap()
                .1,
            ProviderEvent::Started { .. }
        ) {}
        std::thread::sleep(std::time::Duration::from_secs(4));
        let begun = std::time::Instant::now();
        run.cancel().unwrap();
        let events = collect_slow(&rx);
        eprintln!("stop: {:?} after {:?}", events.last(), begun.elapsed());
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: false, .. })
            ),
            "{events:?}"
        );
        assert!(begun.elapsed() < std::time::Duration::from_secs(10));
    }

    fn collect_slow(rx: &std::sync::mpsc::Receiver<(String, ProviderEvent)>) -> Vec<ProviderEvent> {
        let mut events = Vec::new();
        while let Ok((_, event)) = rx.recv_timeout(std::time::Duration::from_secs(180)) {
            let done = matches!(event, ProviderEvent::Finished { .. });
            events.push(event);
            if done {
                break;
            }
        }
        events
    }

    #[test]
    fn stop_ends_the_whole_process_group() {
        let (muse, dir, _spawn) = stand_in("#!/bin/sh\necho started\nsleep 30 &\nwait\n");
        let (tx, rx) = std::sync::mpsc::channel();
        let mut run = muse
            .start("t2", dir.to_str().unwrap(), "x", None, tx)
            .unwrap();
        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_secs(5)),
            Ok((_, ProviderEvent::Started { .. }))
        ));
        // Wait until the script runs; the session hint may come first.
        while rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap()
            .1
            != ProviderEvent::Output("started".into())
        {}
        let begun = std::time::Instant::now();
        run.cancel().unwrap();
        let events = collect(&rx);
        assert!(
            matches!(
                events.last(),
                Some(ProviderEvent::Finished { ok: false, .. })
            ),
            "{events:?}"
        );
        assert!(begun.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn missing_project_folder_is_refused() {
        let (muse, _, _spawn) = stand_in("#!/bin/sh\n");
        let (tx, _rx) = std::sync::mpsc::channel();
        assert!(muse.start("t3", "/no/such/folder", "x", None, tx).is_err());
    }
}
