//! GNOME window focus boundary. Call from a background task, never the UI thread.

use gio::glib::{self, variant::ToVariant};

#[derive(Debug, PartialEq)]
pub enum Activation {
    FocusRequested,
    LaunchRequested,
}

const PATH: &str = "/dev/da/Workstation/Windows";
const INTERFACE: &str = "dev.da.Workstation.Windows";

fn call(method: &str, id: &str) -> Result<glib::Variant, String> {
    call_params(method, Some(&(id,).to_variant()))
}

fn call_params(method: &str, params: Option<&glib::Variant>) -> Result<glib::Variant, String> {
    let connection = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
        .map_err(|error| format!("Desktop connection unavailable: {error}"))?;
    connection
        .call_sync(
            Some("org.gnome.Shell"),
            PATH,
            INTERFACE,
            method,
            params,
            None,
            gio::DBusCallFlags::NO_AUTO_START,
            2000,
            None::<&gio::Cancellable>,
        )
        .map_err(|mut error| {
            let remote = gio::DBusError::remote_error(&error).map(|name| name.to_string());
            gio::DBusError::strip_remote_error(&mut error);
            describe_error(remote.as_deref(), error.message())
        })
}

/// An error the extension raised is a user-facing reason; anything else means it is unreachable.
pub const OLD_EXTENSION: &str = "GNOME is still running the older DA Workstation extension, which lacks this command. Sign out and back in once to load the new one; it also docks the sidebar.";

fn describe_error(remote: Option<&str>, message: &str) -> String {
    if remote.is_some_and(|name| name.starts_with("org.gnome.gjs.JSError.")) {
        message.to_string()
    } else if remote == Some("org.freedesktop.DBus.Error.UnknownMethod")
        && !message.contains("Object does not exist")
    {
        // The extension is loaded, but GNOME loads new extension code only at sign-in.
        // ("Object does not exist" means the extension is off, for example while the
        // screen is locked; that falls through to "unavailable".)
        OLD_EXTENSION.to_string()
    } else {
        format!(
            "Window control unavailable. Enable the DA Workstation GNOME extension. Use Ctrl+Enter for a launch request. ({message})"
        )
    }
}

/// Focus an existing app window, or launch only after GNOME confirms none exists.
pub fn activate(id: &str) -> Result<Activation, String> {
    if gio::DesktopAppInfo::new(id).is_none() {
        return Err("This application is no longer installed. Restart to refresh the list.".into());
    }
    activate_with(
        || {
            call("FocusApp", id)?
                .get::<(bool,)>()
                .map(|(focused,)| focused)
                .ok_or_else(|| {
                    "Unexpected window-control response. Update the GNOME extension.".into()
                })
        },
        || crate::apps::launch(id),
    )
}

fn activate_with(
    focus: impl FnOnce() -> Result<bool, String>,
    launch: impl FnOnce() -> Result<(), String>,
) -> Result<Activation, String> {
    if focus()? {
        Ok(Activation::FocusRequested)
    } else {
        launch()?;
        Ok(Activation::LaunchRequested)
    }
}

/// Return the selected app's window count and whether one currently has focus.
pub fn app_state(id: &str) -> Result<(u32, bool), String> {
    call("GetAppState", id)?
        .get::<(u32, bool)>()
        .ok_or_else(|| "Unexpected window-state response. Update the GNOME extension.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_window_does_not_launch_a_duplicate() {
        assert_eq!(
            activate_with(|| Ok(true), || panic!("must not launch")),
            Ok(Activation::FocusRequested)
        );
    }

    #[test]
    fn confirmed_absence_launches_and_preserves_launch_errors() {
        assert_eq!(
            activate_with(|| Ok(false), || Ok(())),
            Ok(Activation::LaunchRequested)
        );
        assert_eq!(
            activate_with(|| Ok(false), || Err("launch failed".into())),
            Err("launch failed".into())
        );
    }

    #[test]
    fn extension_errors_are_shown_without_the_unavailable_hint() {
        assert_eq!(
            describe_error(Some("org.gnome.gjs.JSError.Error"), "Open two windows"),
            "Open two windows"
        );
        assert_eq!(
            describe_error(Some("org.freedesktop.DBus.Error.UnknownMethod"), "x"),
            OLD_EXTENSION
        );
        assert!(
            describe_error(Some("org.freedesktop.DBus.Error.ServiceUnknown"), "x")
                .starts_with("Window control unavailable")
        );
        assert!(describe_error(None, "timeout").starts_with("Window control unavailable"));
        assert!(
            describe_error(
                Some("org.freedesktop.DBus.Error.UnknownMethod"),
                "Object does not exist at path “/dev/da/Workstation/Windows”"
            )
            .starts_with("Window control unavailable")
        );
    }

    #[test]
    fn unavailable_backend_does_not_launch_a_duplicate() {
        assert_eq!(
            activate_with(
                || Err("extension disabled".into()),
                || panic!("must not launch")
            ),
            Err("extension disabled".into())
        );
    }
}

/// A layout change on the current GNOME monitor and workspace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutRequest {
    /// Tile the most recent windows; `0` tiles all of them.
    Tile(u32),
    Restore,
}

/// Returns how many windows the extension tiled or restored.
pub fn layout(request: LayoutRequest) -> Result<u32, String> {
    let reply = match request {
        // 0 means every window. TileRecent is used for all counts so an old
        // extension (which has only TileWorkspace, for 2–3 windows) reports itself.
        LayoutRequest::Tile(limit) => {
            // GNOME may have moved the sidebar after it opened; dock it first so
            // tiling uses all the space beside it. A missing sidebar is fine.
            // Give the dock move a moment to land so tiling measures the new area.
            if place_sidebar(false).unwrap_or(false) {
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            call_params("TileRecent", Some(&(limit,).to_variant()))
        }
        LayoutRequest::Restore => call_params("RestoreLayout", None),
    }?;
    reply
        .get::<(u32,)>()
        .map(|(count,)| count)
        .ok_or_else(|| "Unexpected layout response. Update the GNOME extension.".into())
}

/// Move focus to the nearest window in a direction: left, right, up or down.
pub fn focus_direction(direction: &str) -> Result<bool, String> {
    if !matches!(direction, "left" | "right" | "up" | "down") {
        return Err(format!(
            "Unknown direction {direction:?}. Use left, right, up or down."
        ));
    }
    call("FocusDirection", direction)?
        .get::<(bool,)>()
        .map(|(focused,)| focused)
        .ok_or_else(|| "Unexpected focus response. Update the GNOME extension.".into())
}

/// One open window, as the sidebar shows it.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenWindow {
    pub id: u64,
    pub title: String,
    pub app: String,
    pub app_id: String,
    pub workspace: i32,
    pub focused: bool,
    pub minimized: bool,
    pub attention: bool,
}

/// Every normal window on every workspace, most recently used first.
pub fn list_windows() -> Result<Vec<OpenWindow>, String> {
    let (json,) = call_params("ListWindows", None)?
        .get::<(String,)>()
        .ok_or("Unexpected window list response. Update the GNOME extension.")?;
    serde_json::from_str(&json).map_err(|error| format!("Unreadable window list: {error}"))
}

pub fn focus_window(id: u64) -> Result<bool, String> {
    call_params("FocusWindow", Some(&(id,).to_variant()))?
        .get::<(bool,)>()
        .map(|(focused,)| focused)
        .ok_or_else(|| "Unexpected focus response. Update the GNOME extension.".into())
}

/// Open an app from DA Workstation into the stage: a new window joins it as the big
/// window (or stages everything when nothing is tiled); an existing window is focused,
/// and stages everything when nothing is tiled. Tiling problems never fail the open.
pub fn open_in_stage(id: &str) -> Result<Activation, String> {
    let _ = stage_next(true);
    let result = activate(id);
    if !matches!(result, Ok(Activation::LaunchRequested)) {
        let _ = stage_next(false);
    }
    if matches!(result, Ok(Activation::FocusRequested)) {
        stage_focused();
    }
    result
}

/// Launch a new window of an app straight into the stage.
pub fn launch_in_stage(id: &str) -> Result<(), String> {
    let _ = stage_next(true);
    crate::apps::launch(id).inspect_err(|_| {
        let _ = stage_next(false);
    })
}

/// After focusing a window from DA Workstation: tile everything (focused window big)
/// when nothing is tiled. When tiled, the focus change already swaps it in.
pub fn stage_focused() {
    if !is_staged() {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let _ = layout(LayoutRequest::Tile(0));
    }
}

/// The next window that opens joins the tiles (for launches DA Workstation starts
/// some other way, such as opening a page in the browser).
pub fn arm_stage() {
    let _ = stage_next(true);
}

fn stage_next(arm: bool) -> Result<bool, String> {
    call_params("StageNext", Some(&(arm,).to_variant()))?
        .get::<(bool,)>()
        .map(|(armed,)| armed)
        .ok_or_else(|| "Unexpected stage response. Update the GNOME extension.".into())
}

/// Auto-tiling: every new normal window joins the stage (on), or only apps opened
/// from DA Workstation (off). Returns the new value.
pub fn set_auto_stage(on: bool) -> Result<bool, String> {
    call_params("SetAutoStage", Some(&(on,).to_variant()))?
        .get::<(bool,)>()
        .map(|(on,)| on)
        .ok_or_else(|| "Unexpected auto-tiling response. Update the GNOME extension.".into())
}

/// The extension's auto-tiling setting; None when the extension does not answer.
pub fn auto_stage() -> Option<bool> {
    call_params("GetLayoutState", None)
        .ok()
        .and_then(|reply| reply.get::<(String,)>())
        .and_then(|(json,)| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|state| state["autoStage"].as_bool())
}

/// True when windows are tiled as a stage right now.
pub fn is_staged() -> bool {
    call_params("GetLayoutState", None)
        .ok()
        .and_then(|reply| reply.get::<(String,)>())
        .and_then(|(json,)| serde_json::from_str::<serde_json::Value>(&json).ok())
        .is_some_and(|state| !state["big"].is_null())
}

/// Where the app docks: a full-height panel on the left, or a wide bar along the bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Left,
    Bottom,
}

impl Placement {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Bottom => "bottom",
        }
    }
}

fn placement_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".local/share"))
        })?;
    Some(base.join("da-workstation/placement"))
}

/// The saved placement; left when none is saved.
pub fn placement() -> Placement {
    match placement_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .as_deref()
        .map(str::trim)
    {
        Some("bottom") => Placement::Bottom,
        _ => Placement::Left,
    }
}

pub fn set_placement(placement: Placement) -> Result<(), String> {
    let path = placement_path().ok_or("No HOME or XDG_DATA_HOME set")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, placement.name()).map_err(|e| format!("Cannot save the placement: {e}"))
}

/// Dock the app at its saved placement. Returns false when no sidebar window exists.
pub fn place_sidebar(activate: bool) -> Result<bool, String> {
    call_params(
        "PlaceSidebarAt",
        Some(&(placement().name(), activate).to_variant()),
    )?
    .get::<(bool,)>()
    .map(|(found,)| found)
    .ok_or_else(|| "Unexpected sidebar response. Update the GNOME extension.".into())
}
