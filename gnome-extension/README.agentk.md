# GNOME window focus

<!-- agent-kit: source_paths gnome-extension/extension.js gnome-extension/metadata.json src/windows.rs -->

For example, select Ghostty in the focus bar. Enter focuses an existing window, or requests a launch when none exists.
This companion extends GNOME Shell 50. Other Shell versions are not verified.
The folder is below the kit's unit-map thresholds; the root map covers it.

## Contract

D-Bus is the desktop's local messaging interface.
The extension exports `dev.da.Workstation.Windows` at `/dev/da/Workstation/Windows` on `org.gnome.Shell`.

| Method | Input | Output |
|---|---|---|
| `FocusApp` | installed desktop ID, string | boolean: a focus request was issued |
| `GetAppState` | installed desktop ID, string | unsigned window count and boolean: an app window has focus |
| `TileWorkspace` | none | unsigned count of tiled windows |
| `RestoreLayout` | none | unsigned count of restored windows |
| `GetLayoutState` | none | JSON string: work area and normal-window rectangles |
| `TileRecent` | unsigned limit (2–9) | unsigned count; minimizes the other windows |
| `FocusDirection` | `left`, `right`, `up` or `down` | boolean: a window was focused |
| `ListWindows` | none | JSON string: every normal window, most recent first |
| `FocusWindow` | unsigned 64-bit window id | boolean: the window exists |
| `PlaceSidebar` | boolean: also raise it | boolean: a sidebar window exists |
| `PlaceSidebarAt` | `left` or `bottom`, boolean: also raise it | boolean: a sidebar window exists |
| `SetAutoStage` | boolean | boolean: auto-tiling on (every new normal window joins the stage); kept in `$XDG_STATE_HOME/da-workstation/autostage` |
| `StageNext` | boolean: arm (true) or cancel | boolean: armed; the next normal window within 20 s joins the stage as the big window |

`FocusApp` uses `Shell.AppSystem.lookup_app`, `Shell.App.get_windows`, and `Main.activateWindow`.
Shell orders windows by workspace, visibility, and recent interaction.
The extension focuses the first window. Workspace switching follows GNOME's window activation behavior.
An unknown desktop ID returns an error. No windows returns false.
The Rust adapter launches only after false. Transport errors and timeouts do not trigger a launch.
The adapter limits each method call to two seconds and runs UI requests on a background thread.
The extension exports no shell command, arbitrary JavaScript evaluation, or close method.
Tiling now adds `TileWorkspace` and `RestoreLayout` (unsigned window counts), plus `GetLayoutState` (JSON containing work area, normal-window rectangles, and `glows`, the number of glow frames).
Each tiled window gets a blue glow frame just behind it (bright on the focused window, faint on the others). Frames follow moves and resizes, hide with their window, and go away on restore, on window close, and when the extension unloads.
Tiling arranges the normal windows on the current monitor/workspace as a stage (D-013): every window gets the big frame; the non-focused ones are drawn scaled into a card column (actor scale and translation; frames and sizes do not change). A reactive `St.Button` over each card in `Main.layoutManager.uiGroup` takes the click and activates the window; a focus change swaps it into the big slot. Cards hide in the overview and on other workspaces. `GetLayoutState` returns `stack`, `big` (window id) and `cards` (id and drawn rectangle).
Ctrl+R restores the saved rectangles and maximization flags. Fullscreen and non-resizable windows cause an error before movement.
The bar is excluded; it remains a floating surface and is not a reserved screen edge.
Restore before tiling again. The snapshot is in memory and is lost when the extension unloads.
Application minimum sizes can constrain placement; oversized minimums and three-window geometry still need acceptance checks.
Other processes on the same user session bus can call this interface while the extension is enabled.
Disabling the extension removes the exported object.

## Installation

The extension bundle was built and installed on October 30 with these commands:

```sh
mkdir -p /tmp/da-workstation-extension
gnome-extensions pack --force --out-dir /tmp/da-workstation-extension gnome-extension
gnome-extensions install /tmp/da-workstation-extension/da-workstation@local.shell-extension.zip
```

If GNOME does not recognize a newly installed extension, save work, then sign out and sign in.
Enable **DA Workstation Window Control** in the Extensions app, or run:

```sh
gnome-extensions enable da-workstation@local
```

The initial session returned “does not exist”. On continuation October 30, GNOME recognized the extension;
enabling succeeded and `gnome-extensions info da-workstation@local` reported `Enabled: Yes` and `State: ACTIVE`.
Do not restart or replace the user's running compositor to load the extension.

## Verification

The real isolated GNOME test passes. See [the report](../experiments/gnome-focus/REPORT.agentk.md).
The current user desktop also accepts the Rust `--activate` command for Ghostty. State checks report one focused window.
The user confirms search typing, selection, and Enter focus an open Ghostty window without creating another window.
Check an app with no windows, extension disabled, minimized windows, and a window on another workspace.
The last two cases remain unverified. App identity depends on GNOME associating windows with the selected desktop entry.

References: [GNOME 50 D-Bus source](https://raw.githubusercontent.com/GNOME/gnome-shell/50.6/js/ui/shellDBus.js),
[Shell window ordering](https://gnome.pages.gitlab.gnome.org/gnome-shell/shell/method.App.get_windows.html),
and [extension development](https://gjs.guide/extensions/development/creating.html).

## Tiling update, October 30

The updated bundle is installed. GNOME exposes `ReloadExtension` in introspection but returns “not implemented” when called.
Save work and sign out/in to load the new code; then close/reopen the focus bar to use the rebuilt binary.
The existing session still runs the previous focus-only extension. No compositor restart was attempted.

## November 2 update

Pure geometry lives in `layout.js`; `experiments/gnome-focus/layout_test.js` tests it with `gjs -m`.
Pack with `--extra-source=layout.js`, or GNOME loads the extension without it:

```sh
gnome-extensions pack --force --extra-source=layout.js --out-dir /tmp/da-workstation-extension gnome-extension
gnome-extensions install --force /tmp/da-workstation-extension/da-workstation@local.shell-extension.zip
```

GNOME 50 (Mutter 18) `maximize()` and `unmaximize()` take no flags; the extension uses the axis methods for one-axis restore.
The restore snapshot is saved in `$XDG_STATE_HOME/da-workstation/layout.json` with window ids and the Shell process id.
The headless harness passes 21 checks (it uses its own `XDG_STATE_HOME` since November 2) (glow frames, stage focus, auto-stage and bottom bar added November 2), including four windows, `--tile 2`, directional focus, restore after reload, and the docked sidebar.
The bundle is installed; it loads at the next sign-out/in.
