//! The Help page: what DA Workstation is and everything it can do, by category. It is
//! written fresh as HTML each time (the command and Muse tool lists come from the code
//! that runs them) and opened in the default browser, which then becomes the big window.

use crate::{mcp, switcher, tasks::TaskState, windows};
use gio::prelude::*;

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// One category: a heading, an optional note, and (key, explanation) rows.
fn group(title: &str, note: &str, rows: &[(String, String)]) -> String {
    let rows: String = rows
        .iter()
        .map(|(key, about)| format!("<tr><th>{}</th><td>{}</td></tr>", key, esc(about)))
        .collect();
    let note = if note.is_empty() {
        String::new()
    } else {
        format!("<p class=\"note\">{}</p>", esc(note))
    };
    format!(
        "<section><h2>{}</h2>{note}<table>{rows}</table></section>",
        esc(title)
    )
}

/// Keys as `<kbd>`: "Ctrl+Alt+Space" becomes three keys.
fn keys(combo: &str) -> String {
    combo
        .split(" / ")
        .map(|one| {
            one.split('+')
                .map(|k| format!("<kbd>{}</kbd>", esc(k)))
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn rows(list: &[(&str, &str)], as_keys: bool) -> Vec<(String, String)> {
    list.iter()
        .map(|(k, a)| {
            let key = if as_keys { keys(k) } else { esc(k) };
            (key, a.to_string())
        })
        .collect()
}

pub fn html() -> String {
    let commands: Vec<(String, String)> = switcher::Command::ALL
        .iter()
        .map(|c| {
            let (name, about) = c.text();
            (format!("<code>/{}</code>", esc(name)), about.to_string())
        })
        .collect();
    let tools: Vec<(String, String)> = mcp::tool_summaries()
        .into_iter()
        .map(|(name, about)| (esc(&name), about))
        .collect();
    let states: Vec<(String, String)> = [
        (TaskState::Running, "Muse is working. Ctrl+. stops it."),
        (
            TaskState::NeedsPermission,
            "Muse waits for your Allow once or Reject.",
        ),
        (
            TaskState::Done,
            "The run ended. Check the result: Muse's last message is not proof it worked.",
        ),
        (
            TaskState::Failed,
            "The run failed. Read the error, then send a follow-up to retry.",
        ),
        (
            TaskState::Stopped,
            "You stopped it. A follow-up continues the same conversation.",
        ),
        (
            TaskState::Interrupted,
            "The app restarted during a run; its outcome is unknown until you check.",
        ),
    ]
    .into_iter()
    .map(|(state, about)| {
        let (icon, label) = state.label();
        (esc(&format!("{icon} {label}")), about.to_string())
    })
    .collect();

    let sections = [
        group(
            "How to use it",
            "",
            &rows(
                &[
                    ("1. Open", "Press Ctrl+Alt+Space anywhere. The panel opens with the typing box ready."),
                    ("2. Find", "Type a name (\"chrome\", \"files\", a task title). Enter opens the top match; new windows tile themselves."),
                    ("3. Ask Muse", "Type a sentence and press Enter: it becomes a task. Start with @~/folder to choose the project folder."),
                    ("4. Answer", "When Muse wants to run a command or change a file, choose Allow once or Reject, in the task or under \"Needs you\"."),
                    ("5. Follow up", "Open a task and type a reply. Muse keeps the conversation and its context."),
                ],
                false,
            ),
        ),
        group(
            "Shortcuts anywhere",
            "These work in every app.",
            &rows(
                &[
                    ("Ctrl+Alt+Space", "Open the panel with the typing box focused"),
                    ("Ctrl+Alt+B", "Open or raise the panel"),
                    ("Ctrl+Super+T", "Tile windows: one big, the others as cards"),
                    ("Ctrl+Super+R", "Put tiled windows back where they were"),
                    ("Ctrl+Super+Left / Ctrl+Super+Right", "Move to the previous or next window"),
                ],
                true,
            ),
        ),
        group(
            "Shortcuts in the panel",
            "",
            &rows(
                &[
                    ("Enter", "Run the chosen row, start a task, or send a follow-up"),
                    ("Up / Down", "Choose a row under the typing box"),
                    ("Escape", "Clear the box, or go back to the task list"),
                    ("Ctrl+K", "Back to the task list with the box focused"),
                    ("Ctrl+.", "Stop the running task"),
                    ("Ctrl+T / Ctrl+R", "Tile / restore (while the panel has focus)"),
                    ("Ctrl+2 / Ctrl+9", "Tile only that many recent windows (any number 2 to 9)"),
                    ("Ctrl+/", "Open this page"),
                ],
                true,
            ),
        ),
        group(
            "The typing box",
            "One box does everything. What Enter does depends on what you type.",
            &rows(
                &[
                    ("A few words", "Matching tasks, open apps and installed apps come first; Enter opens the top one"),
                    ("A sentence", "\"Ask Muse\" comes first; Enter starts a task"),
                    ("@~/folder …", "Start the task in that project folder (it becomes the default)"),
                    ("/", "List the commands below"),
                ],
                false,
            ),
        ),
        group("Commands", "Type / and the start of a name.", &commands),
        group(
            "Windows and tiling",
            "",
            &rows(
                &[
                    ("Big window", "The window you work in, beside the panel, at a desktop size"),
                    ("Cards", "Your other windows on the right, small and live, captioned with the app and its title (a browser's open page)"),
                    ("Click a card", "Swap it with the big window; no window is resized"),
                    ("Auto-tile on", "Every new window joins the tiles, from any launcher (switch under the box, or /autotile off)"),
                    ("Auto-tile off", "Only apps opened from DA Workstation tile"),
                    ("Bottom bar", "Dock the panel along the bottom instead of the left (switch under the box)"),
                ],
                false,
            ),
        ),
        group(
            "Muse does these without asking",
            "Desktop tools DA Workstation gives Muse in every task. Ask in plain words: \"open Chrome and make it the big window\".",
            &tools,
        ),
        group(
            "Muse asks you first",
            "",
            &rows(
                &[
                    ("Shell commands", "Each command. A command with several steps (a && b) needs one answer"),
                    ("File changes", "Writing or editing files in the project folder"),
                    ("Not yet possible", "Reading or clicking inside web pages; Muse can only open them"),
                ],
                false,
            ),
        ),
        group("Task states", "Each task shows an icon and a word; color is never the only sign.", &states),
        group(
            "Where things are kept",
            "",
            &rows(
                &[
                    ("Tasks", "~/.local/share/da-workstation/tasks, one file per task; they survive restarts"),
                    ("Muse session", "Shown in each task; `muse resume <id>` in that folder opens it in a terminal"),
                    ("Settings", "Panel placement in ~/.local/share/da-workstation; auto-tile in ~/.local/state/da-workstation"),
                ],
                false,
            ),
        ),
    ]
    .concat();

    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>DA Workstation Help</title>
<style>
:root {{ --bg:#0b0b0f; --panel:#15151b; --border:#2a2a35; --text:#f4f4f5; --muted:#8e8e9a; --accent:#38bdf8; --warn:#fbbf24; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; background:var(--bg); color:var(--text); font:15px/1.55 system-ui, sans-serif; }}
main {{ max-width:880px; margin:0 auto; padding:40px 20px 80px; }}
header {{ display:flex; align-items:center; gap:12px; margin-bottom:8px; }}
.dot {{ width:12px; height:12px; border-radius:50%; background:var(--accent); }}
h1 {{ font-size:26px; margin:0; }}
.lead {{ color:#d4d4d8; font-size:16px; margin:12px 0 28px; }}
nav {{ display:flex; flex-wrap:wrap; gap:8px; margin-bottom:28px; }}
nav a {{ color:var(--accent); text-decoration:none; border:1px solid var(--border); border-radius:999px; padding:3px 12px; font-size:13px; }}
nav a:hover {{ background:var(--panel); }}
section {{ background:var(--panel); border:1px solid var(--border); border-radius:12px; padding:18px 20px; margin-bottom:16px; }}
h2 {{ font-size:13px; letter-spacing:.06em; text-transform:uppercase; color:var(--muted); margin:0 0 10px; }}
.note {{ color:var(--muted); margin:0 0 10px; font-size:14px; }}
table {{ width:100%; border-collapse:collapse; }}
th {{ text-align:left; vertical-align:top; width:230px; padding:7px 14px 7px 0; font-weight:600; color:var(--accent); font-size:14px; }}
td {{ padding:7px 0; color:#d4d4d8; font-size:14px; border-top:1px solid #1f1f27; }}
tr:first-child td {{ border-top:none; }}
kbd {{ background:#1d1d25; border:1px solid var(--border); border-bottom-width:2px; border-radius:6px; padding:1px 6px; font:12px ui-monospace, monospace; color:var(--text); }}
code {{ font:13px ui-monospace, monospace; color:var(--accent); }}
footer {{ color:var(--muted); font-size:13px; margin-top:24px; }}
@media (max-width:600px) {{ th {{ width:120px; }} }}
</style></head>
<body><main>
<header><div class="dot"></div><h1>DA Workstation</h1></header>
<p class="lead">One panel for your desktop and your coding agent. Your windows tile around your work: one big window, the others as live cards on the right. Muse Code, Meta's coding agent, runs tasks in your project folders; each task keeps its chat, results and approvals in the panel, and Muse can also open apps, tile windows and open web pages for you.</p>
<nav>{nav}</nav>
{sections}
<footer>This page is written fresh each time you open it from the panel (the ? button, /help or Ctrl+/), so the lists match the version you run.</footer>
</main></body></html>
"#,
        nav = [
            "How to use it", "Shortcuts anywhere", "Shortcuts in the panel", "The typing box",
            "Commands", "Windows and tiling", "Muse does these without asking", "Muse asks you first",
            "Task states",
        ]
        .iter()
        .map(|t| format!("<a href=\"#{}\">{}</a>", slug(t), esc(t)))
        .collect::<String>(),
    )
    .replace("<section><h2>", "<section><h2 id=\"\">")
    .split("<h2 id=\"\">")
    .enumerate()
    .map(|(i, part)| {
        if i == 0 {
            part.to_string()
        } else {
            let title = part.split('<').next().unwrap_or_default();
            format!("<h2 id=\"{}\">{part}", slug(&title.replace("&amp;", "&")))
        }
    })
    .collect()
}

fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Write the page and open it in the default browser: a new tab when the browser is
/// open (then it becomes the big window), or a new browser window that tiles itself.
pub fn open() -> Result<String, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".local/share")))
        .ok_or("No HOME or XDG_DATA_HOME set")?;
    let path = base.join("da-workstation/help.html");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, html()).map_err(|e| format!("Cannot write the Help page: {e}"))?;
    let uri = gio::File::for_path(&path).uri().to_string();
    let browser = gio::AppInfo::default_for_type("text/html", false)
        .and_then(|app| app.id())
        .map(|id| id.to_string());
    let open_already = browser
        .as_deref()
        .and_then(|id| windows::app_state(id).ok())
        .is_some_and(|(count, _)| count > 0);
    if !open_already {
        windows::arm_stage();
    }
    gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>)
        .map_err(|e| format!("Could not open the Help page: {e}"))?;
    if let (true, Some(id)) = (open_already, browser) {
        std::thread::sleep(std::time::Duration::from_millis(400));
        if windows::activate(&id).is_ok() {
            windows::stage_focused();
        }
    }
    Ok(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_lists_every_command_and_tool_with_anchors() {
        let page = html();
        for command in switcher::Command::ALL {
            assert!(
                page.contains(&format!("/{}", command.text().0)),
                "{command:?}"
            );
        }
        for (name, _) in mcp::tool_summaries() {
            assert!(page.contains(&name), "{name}");
        }
        assert!(page.contains("id=\"commands\"") && page.contains("href=\"#commands\""));
        assert!(page.contains("<kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>Space</kbd>"));
        assert!(!page.contains("<script"));
    }
}
