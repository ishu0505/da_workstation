//! DA Workstation tools for agents: a small MCP server on stdio (`da-workstation --mcp`,
//! one JSON-RPC message per line). Muse gets it per session; its calls are answered
//! without asking the user (only these tools; shell commands and file edits still ask).

use crate::{apps, windows};
use serde_json::{Value, json};
use std::io::{BufRead, Write};

/// The MCP server name given to Muse; its tools appear as `mcp__<SERVER>__<tool>`.
pub const SERVER: &str = "da_workstation";

/// True for a Muse tool name that belongs to this server.
pub fn is_own_tool(name: &str) -> bool {
    name.starts_with(&format!("mcp__{SERVER}__")) || name.starts_with(&format!("{SERVER}__"))
}

/// Tool names and descriptions, for the Help page.
pub fn tool_summaries() -> Vec<(String, String)> {
    tools()
        .as_array()
        .into_iter()
        .flatten()
        .map(|t| {
            (
                t["name"].as_str().unwrap_or_default().replace('_', " "),
                t["description"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

fn tools() -> Value {
    let none = json!({"type": "object", "properties": {}});
    json!([
        {"name": "list_windows",
         "description": "List every open normal window: id, title, app, app id, workspace, focused, minimized.",
         "inputSchema": none},
        {"name": "find_apps",
         "description": "Search installed apps by name. Returns app ids for open_app and new_window.",
         "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}},
        {"name": "open_app",
         "description": "Focus an app's open window, or launch it when it has none, and make it the big tiled window (windows are tiled automatically). Takes an app id from find_apps (for example google-chrome.desktop) or an app name.",
         "inputSchema": {"type": "object", "properties": {"app": {"type": "string"}}, "required": ["app"]}},
        {"name": "new_window",
         "description": "Launch a new window of an app, even when one is open.",
         "inputSchema": {"type": "object", "properties": {"app": {"type": "string"}}, "required": ["app"]}},
        {"name": "focus_window",
         "description": "Focus one window by its id from list_windows.",
         "inputSchema": {"type": "object", "properties": {"id": {"type": "integer"}}, "required": ["id"]}},
        {"name": "tile",
         "description": "Tile windows on the current workspace: one big window and the others as small cards beside it. count 0 tiles all; 2 to 9 tiles that many recent windows and minimizes the rest. Focus a window first (focus_window) to make it the big one. Call restore_layout before tiling again.",
         "inputSchema": {"type": "object", "properties": {"count": {"type": "integer", "minimum": 0, "maximum": 9}}}},
        {"name": "restore_layout",
         "description": "Undo tiling: put windows back where they were.",
         "inputSchema": none},
        {"name": "focus_direction",
         "description": "Move focus to the next window: left, right, up or down.",
         "inputSchema": {"type": "object", "properties": {"direction": {"type": "string", "enum": ["left", "right", "up", "down"]}}, "required": ["direction"]}},
        {"name": "open_url",
         "description": "Open a web address in the default browser.",
         "inputSchema": {"type": "object", "properties": {"url": {"type": "string"}}, "required": ["url"]}}
    ])
}

/// Resolve an app id, or the best name match among installed apps.
fn app_id(name: &str) -> Result<String, String> {
    if name.ends_with(".desktop") {
        return Ok(name.to_string());
    }
    let installed = apps::discover();
    apps::matches(&installed, name)
        .first()
        .map(|&i| installed[i].id.clone())
        .ok_or_else(|| format!("No installed app matches {name:?}. Try find_apps."))
}

/// Run one tool. Ok is the text result for the agent; Err becomes an error result.
pub fn call(name: &str, args: &Value) -> Result<String, String> {
    let text = |key: &str| {
        args[key]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(format!("Missing argument {key:?}."))
    };
    match name {
        "list_windows" => {
            let list: Vec<Value> = windows::list_windows()?
                .into_iter()
                .map(|w| {
                    json!({"id": w.id, "title": w.title, "app": w.app, "appId": w.app_id,
                           "workspace": w.workspace + 1, "focused": w.focused, "minimized": w.minimized})
                })
                .collect();
            Ok(serde_json::to_string_pretty(&list).unwrap_or_default())
        }
        "find_apps" => {
            let installed = apps::discover();
            let found: Vec<Value> = apps::matches(&installed, text("query")?)
                .into_iter()
                .take(10)
                .map(|i| json!({"id": installed[i].id, "name": installed[i].name}))
                .collect();
            Ok(serde_json::to_string_pretty(&found).unwrap_or_default())
        }
        "open_app" => {
            let id = app_id(text("app")?)?;
            Ok(match windows::activate(&id)? {
                windows::Activation::FocusRequested => format!("Focused {id}."),
                windows::Activation::LaunchRequested => {
                    format!("Launched {id} (it had no window).")
                }
            })
        }
        "new_window" => {
            let id = app_id(text("app")?)?;
            windows::launch_in_stage(&id)?;
            Ok(format!(
                "Launched a new window of {id}; it joins the tiles as the big window."
            ))
        }
        "focus_window" => {
            let id = args["id"].as_u64().ok_or("Missing argument \"id\".")?;
            if windows::focus_window(id)? {
                Ok(format!("Focused window {id}."))
            } else {
                Err(format!(
                    "No window {id}. Call list_windows for current ids."
                ))
            }
        }
        "tile" => {
            let count = args["count"].as_u64().unwrap_or(0).min(9) as u32;
            let count = if count == 1 { 2 } else { count };
            let n = windows::layout(windows::LayoutRequest::Tile(count))?;
            Ok(format!(
                "Tiled {n} windows: the focused one big, the others as cards."
            ))
        }
        "restore_layout" => {
            let n = windows::layout(windows::LayoutRequest::Restore)?;
            Ok(format!("Restored {n} windows."))
        }
        "focus_direction" => {
            let moved = windows::focus_direction(text("direction")?)?;
            Ok(if moved {
                "Focus moved."
            } else {
                "No window in that direction."
            }
            .into())
        }
        "open_url" => {
            let url = text("url")?;
            let url = if url.contains("://") {
                url.to_string()
            } else {
                format!("https://{url}")
            };
            gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>)
                .map_err(|e| format!("Could not open {url}: {e}"))?;
            Ok(format!("Opened {url} in the default browser."))
        }
        _ => Err(format!("Unknown tool {name:?}.")),
    }
}

/// Answer one JSON-RPC message; None for notifications.
pub fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let result = match msg["method"].as_str().unwrap_or_default() {
        "initialize" => json!({
            "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER, "version": env!("CARGO_PKG_VERSION")},
            "instructions": "Controls the user's GNOME desktop through DA Workstation: open and focus apps, list windows, tile and restore, open web pages. Use these tools instead of shell commands for desktop actions."
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools()}),
        "tools/call" => {
            let name = msg["params"]["name"].as_str().unwrap_or_default();
            let args = &msg["params"]["arguments"];
            let (text, error) = match call(name, args) {
                Ok(text) => (text, false),
                Err(text) => (text, true),
            };
            json!({"content": [{"type": "text", "text": text}], "isError": error})
        }
        method => {
            return Some(json!({"jsonrpc": "2.0", "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}}));
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Serve on stdin/stdout until stdin closes.
pub fn serve() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some(reply) = handle(&msg) {
            let _ = writeln!(out, "{reply}");
            let _ = out.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_tools_and_reports_errors_as_tool_results() {
        let init = handle(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": "2025-06-18"}}))
        .unwrap();
        assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
        assert!(
            handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none()
        );
        let list = handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).unwrap();
        let names: Vec<&str> = list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(
            names.contains(&"open_app") && names.contains(&"tile") && names.contains(&"open_url")
        );
        let bad = handle(&json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": {"name": "focus_window", "arguments": {}}}))
        .unwrap();
        assert_eq!(bad["result"]["isError"], true);
        assert!(is_own_tool("mcp__da_workstation__open_app"));
        assert!(!is_own_tool("bash"));
    }
}
