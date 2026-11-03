# GNOME focus evidence

<!-- agent-kit: source_paths experiments/gnome-focus/verify.py src/windows.rs gnome-extension/extension.js -->

Question: can the Rust launcher focus an existing native app window on GNOME Wayland without creating another window?
Decision: [D-007](../../docs/decisions/D-007-gnome-focus.agentk.md).

## Environment and method

October 30, 2026, Asia/Singapore: Fedora 44, GNOME Shell/Mutter 50.6, Wayland, Rust 1.99.1.
`verify.py` starts a separate session bus and a headless GNOME compositor with a 1280×800 virtual monitor.
Temporary data and settings directories contain the extension and two GTK 4 fixture desktop apps.
The fixture apps are real native windows with fixed test IDs; no model or mocked focus response supplies the result.
The expected result is one window per fixture and a true focus state after explicit activation.
The script stops its compositor when finished. It does not replace or restart the user's compositor.

## Results

The first completed focus run passes these checks:

| Check | Observed result |
|---|---|
| Extension load | GNOME exports `dev.da.Workstation.Windows` |
| App A has no windows | Rust requests launch; GNOME reports one window |
| App B has no windows | Rust requests launch; GNOME reports one window |
| Activate existing app A | Rust reports focus requested; GNOME reports `windows=1 focused=true` |
| Activate app A again | GNOME still reports one focused window |

The saved test passes an explicit B-to-A focus change and extension disable/re-enable recovery, with exit status 0.
Disabling the extension returns an error from activation. Re-enabling confirms app A still has one window.
Final GNOME log: `/tmp/da-focus-check-1vntr1n_/shell.log`.
Final bus diagnostics: `/tmp/da-focus-bus-ve2jwjnx/dbus.log`.
These temporary logs are not required for reproduction.

Six Rust tests pass, including focus without launch, confirmed absence, launch errors, and unavailable-backend behavior.
The real user desktop rejects built-in `GetRunningApplications` and `FocusApp` calls with `AccessDenied`.
The new extension installs successfully, but enabling it before a new desktop session reports “does not exist”.
The Rust adapter reports the absent object as an error and does not launch automatically.

### Current desktop continuation, October 30

GNOME now recognizes the installed extension. `gnome-extensions enable da-workstation@local` succeeds;
extension information reports `Enabled: Yes` and `State: ACTIVE`.
The Rust `--app-state com.mitchellh.ghostty.desktop` command reports `windows=1 focused=true`.
`--activate com.mitchellh.ghostty.desktop` reports a focus request; later state checks still report one focused window.
Ghostty was already focused before the command, so this checks the live connection and absence of a duplicate,
not a focus change from another application. The isolated test above proves that change with fixture apps.
The user confirms typing `terminal`, selecting Ghostty, and pressing Enter in the focus bar focus the existing window without opening another.

## Reproduction

Build the binary, then run the saved integration test:

```sh
cargo build --locked --offline
python3 experiments/gnome-focus/verify.py
```

The test needs GNOME Shell, GJS, GTK 4, `gsettings`, `gdbus`, `dbus-run-session`, and compositor device access.
It prints pass results and paths to temporary diagnostic logs. Desktop services can emit unrelated diagnostics in that isolated bus.
The test does not assume that a newly launched app automatically receives focus; that depends on compositor activation policy.

## Limits

This proves the Rust-to-extension path and real native fixture focus on GNOME 50.
The user confirms Ghostty focus through the bar and search typing. Clipboard editing, minimized windows, and cross-workspace activation remain manual checks.
Global summon, macOS integration, and Muse execution are not measured by this isolated test.
The first temporary harness waited on output inherited by a launched fixture; using files for captured output resolved that harness issue.

## Tiling continuation, October 30

The harness now requests tiling and reads actual rectangles through `GetLayoutState`.
Two real GTK windows remain within the monitor work area and do not overlap. Restore returns both to their exact original rectangles.
Final successful log: `/tmp/da-focus-check-hqxbv5wz/shell.log`. Existing focus and extension-disable checks still pass.
The first run caught a removed `get_maximized` method; the implementation uses GNOME 50 `get_maximize_flags`.
Three-window placement, application minimum-size constraints, maximized-window recovery, and desktop Ctrl+T/Ctrl+R remain unverified.
