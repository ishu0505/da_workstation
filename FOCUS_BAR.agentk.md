# First focus bar

<!-- agent-kit: source_paths Cargo.toml src/main.rs src/apps.rs src/text_input.rs src/windows.rs gnome-extension/extension.js -->

The October 30 slice is a native Rust launcher on Fedora GNOME Wayland.
For example, search for `terminal`, select Ghostty, and press Enter to focus it or request a terminal window.

## Files

| Path | Purpose |
|---|---|
| `src/main.rs` | GPUI window, keyboard selection, and launch feedback |
| `src/apps.rs` | Installed desktop app discovery, matching, and launch boundary |
| `src/windows.rs` | GNOME focus and layout adapter: focus-before-launch, tile, restore |
| `src/sidebar.rs` | Left sidebar: task chats, open apps, alerts (`--sidebar`) |
| `src/tasks.rs` | Task model and JSON store; restart marks active runs interrupted |
| `src/provider.rs`, `src/provider/muse.rs` | Provider boundary and the Muse adapter |
| `src/notify.rs` | Desktop notifications with a click that opens the task |
| `src/text_input.rs` | Native text input adapted from GPUI's Apache-2.0 example |
| `gnome-extension/` | GNOME 50 companion; setup in `gnome-extension/README.agentk.md` |
| `scripts/install-shortcut.py` | Installs the desktop entry and GNOME shortcuts: Ctrl+Alt+Space, Ctrl+Alt+B, Ctrl+Super+arrows |
| `Cargo.toml` | Pinned GPUI and GIO dependencies |
| `Cargo.lock` | Resolved dependency versions |
| `LICENSE-GPUI` | Upstream license for the adapted input example |

This is one small application, below the kit's unit-map thresholds. The root map covers its source files.

## Behavior

- Search app names and desktop metadata. `use-w` is an optional search prefix.
- Press Up/Down or Tab/Shift+Tab to select a result. Enter or a click focuses an existing app window, or launches when none exists.
- Ctrl+Enter requests a launch directly. Apps decide whether that creates a new window.
- If the GNOME extension is unavailable, show an error and offer Ctrl+Enter. An unknown window state does not trigger an automatic launch.
- Press Ctrl+L to clear. Use Ctrl+A/C/V/X for text editing. Press Escape to close the launcher.
- Show focus and launch errors. Successful feedback means a request was accepted; actual focus or window creation requires observation.
- Search text never runs through a shell. Only selected installed desktop entries can launch.
- Run launch requests outside the UI thread. Ignore repeated launch input while a request is pending.

## Boundaries

Existing-window focus requires the GNOME 50 extension. The first tiling implementation is available after loading the updated extension; the hollow frame, persistent chats, and Muse remain subsequent work.
The launcher uses Linux desktop integration. This build does not establish macOS support.
The app list refreshes on restart. Search uses ranked text matching; frecency and fuzzy matching come later.

## Verification

October 30 environment: Fedora 44, GNOME Wayland, Rust 1.99.1, and Ghostty.

- The user installed `libxkbcommon-x11-devel` and `rustfmt`, resolving the previous linker blocker.
- `cargo build --locked --offline` passes. All seven search, activation, and error-message tests pass with `cargo test --locked --offline`.
- `cargo fmt --check`, kit checks, and `git diff --check` pass after formatting the Rust source.
- `target/debug/da-workstation --list terminal` finds GNOME Terminal, Ghostty, and kitty.
- A launch request for a missing desktop ID exits with status 1 and a clear error.
- `target/debug/da-workstation` starts without diagnostic output. The user confirms that the native focus bar appears.
- `target/debug/da-workstation --launch com.mitchellh.ghostty.desktop` succeeds. The user confirms that Ghostty opens.
- Real isolated GNOME 50 checks prove app launch and existing-window focus without duplicate windows; see [the report](experiments/gnome-focus/REPORT.agentk.md).
- The companion extension is installed and active in the current user desktop. `--activate com.mitchellh.ghostty.desktop` succeeds; subsequent state checks report `windows=1 focused=true`; see [setup](gnome-extension/README.agentk.md).
- The user confirms typing `terminal`, selecting Ghostty, and pressing Enter focus the existing window without opening another. Clipboard editing, Escape, and a launch from the UI with no existing window remain manual checks.
- Cargo reports a future Rust compatibility warning in the transitive dependency `proc-macro-error2` 2.0.1. It does not prevent this build or its tests.

Run the focus bar from the repository root:

```sh
target/debug/da-workstation
```

GPUI 0.2.2 pulls in old `xattr` through its HTTP library. Pin `libc` to 0.2.186 to preserve the Linux constant that package expects.
Keep `Cargo.lock`. Revisit this compatibility pin when upgrading GPUI.

The core launcher/focus slice is complete in the tracker. Retain the remaining manual checks above for broader UI verification.

## Global summon

`python3 scripts/install-shortcut.py` installs a desktop entry for this checkout's debug binary and a GNOME custom shortcut, Ctrl+Alt+Space.
Build the binary first. The installer requires Python GObject bindings and preserves other custom shortcuts.
The shortcut uses `--activate dev.da.Workstation.desktop`: focus the existing bar, or launch when GNOME confirms no window exists.
Escape closes the bar; the shortcut can request a fresh launch. The extension must remain enabled.
The binary path points into this checkout; rerun setup if the checkout moves.

October 30: the initial Ctrl+Super+Space binding did not work for the user. Changed Linux to Ctrl+Alt+Space at the user’s request; the saved GNOME setting is verified. No conflict appeared in the inspected GNOME bindings.
Installation succeeds and the registered command reports `Focus requested: dev.da.Workstation.desktop`.
The user confirms Ctrl+Alt+Space brings the bar forward after removing and re-registering its custom shortcut. No logout was needed. The cause of the initial registration failure is not established. The user also confirms Ctrl+Alt+Space reopens the bar after Escape.
The user prefers Ctrl+Command+Space for macOS; registration and conflict checks on macOS remain unimplemented.

To remove the binding, delete **DA Workstation Focus Bar** in GNOME Settings → Keyboard → Custom Shortcuts.
The desktop entry is `~/.local/share/applications/dev.da.Workstation.desktop`.

## First tiling increment

November 2 update: summon the bar and press Ctrl+T to tile every normal window on the current monitor/workspace (two or more).
Ctrl+2 to Ctrl+9 tile that many of the most recent windows and minimize the others.
Auto-tiling (D-015, on by default): every new window, from DA Workstation, Muse or the GNOME menu, becomes the big window (and everything is staged if nothing was). Dialogs and other workspaces are left alone. With auto-tiling off, only apps opened from DA Workstation tile (D-014). Switch it in the panel footer or with /autotile on|off. Each card on the right shows the app and its window title (for a browser, the open page).
Stage (D-013): one big window beside the sidebar; the others are small cards in a column on the right (the same windows drawn scaled, so they keep desktop proportions). Click a card to swap it with the big window; Ctrl+Super+arrows cycle. Desktop-wide: Ctrl+Super+T tiles, Ctrl+Super+R restores (plain Ctrl+T stays with apps; in the sidebar Ctrl+T also works).
Press Ctrl+R to restore; it also un-minimizes. The snapshot survives an extension reload, not a sign-out.
Ctrl+Super+arrows move focus to the nearest window in that direction (`--focus left|right|up|down`). The bar floats above the layout; dismiss it with Escape.
CLI equivalents are `target/debug/da-workstation --tile [2-9]` and `target/debug/da-workstation --restore-layout`.
Requests run off the UI thread. Feedback confirms a request, not final placement.
November 1: after sign-out/in, the user confirms Ctrl+T tiles and Ctrl+R restores in the real desktop.
Tiled windows get a blue glow frame (bright on the focused window); Ctrl+R removes it. The updated extension needs one sign-out/in.
Errors raised by the extension show as-is, for example “Open two or three normal windows on this monitor and workspace”. Only an unreachable extension shows the “Window control unavailable” hint. Ctrl+R with no saved layout says so.
See [extension setup and limits](gnome-extension/README.agentk.md). The updated extension needs sign-out/in on this GNOME session.

## Sidebar and Muse tasks

The sidebar is the whole app (D-012) with one typing box (D-013). The header button switches between the side panel and a bottom bar (D-014; full width, 300 px, three columns: sessions, chat/results with the box, apps); the choice is remembered and tiling uses the space beside or above it. Ctrl+Alt+Space opens it with the box focused (`--sidebar --search`); Ctrl+K returns to it. On the home view, typing lists matching sessions, open apps and installed apps plus "Ask Muse": a short query puts matches first ("chrome" + Enter focuses Chrome), a sentence or `@folder …` puts Ask Muse first. Up/Down choose, Enter runs, Escape or × clears. In a chat the box sends follow-ups.
Muse can drive the desktop (D-013): each session gets the workstation's tools (`da-workstation --mcp`: list windows, find/open apps, new window, focus, tile, restore, move focus, open a URL), answered without asking you. Shell commands and file edits still ask. Ctrl+T, Ctrl+2–9 and Ctrl+R tile and restore from the sidebar. The floating bar (no arguments) still exists but no shortcut opens it; `python3 scripts/install-shortcut.py` sets Ctrl+Alt+Space.
Help: the ? button in the panel header, `/help` or Ctrl+/ writes a fresh guide (`src/help.rs`: what DA Workstation is, how to use it, shortcuts, the typing box, commands, tiling, what Muse does with and without asking, task states, where things are kept) to `~/.local/share/da-workstation/help.html` and opens it in the default browser (a new tab when it is open; the browser becomes the big window). `da-workstation --help-html` prints the page.
Home (D-015, after the Claude and ChatGPT apps): "＋ New task", a "Needs you" box only while a request waits (Allow once / Reject in place), then sessions by day (Today, Yesterday, Previous 7 days, Older folded). Open apps are the cards on the right, not a list here. Type `/` for commands: /tile, /restore, /bottom, /side, /autotile on, /autotile off. The footer under the box shows the project folder and the side/bottom and auto-tile switches. Chats show messages with avatar, name and time; tool steps are quiet lines.

Press Ctrl+Alt+B, or Ctrl+B in the bar, to open the sidebar (`target/debug/da-workstation --sidebar`). A second request raises the open one.
The GNOME extension docks it on the left at full height; tiling leaves its width free.
GNOME can move a new window after its first placement (seen at login), so the sidebar docks again 1.5 s after start, and Ctrl+T docks it before tiling.
Tasks: type a goal and press Enter to start a Muse task. The folder is the newest task's folder (else the folder the sidebar started in); the hint under the input names it. Start with `@~/folder` to choose another; a missing folder gives an alert.
At the top of a task, a card shows the goal, the files Muse changed (newest first, relative to the folder), the outcome with the last answer, and the next action. Each run's end line lists the files that run changed.
The focus bar lists up to three tasks above the apps (running ones first; typing filters by title and goal). Enter on a task opens it in the sidebar (`--sidebar --open TASK_ID`), starting the sidebar if needed.
Each task keeps its chat, status (icon and text), run count and session reference in `~/.local/share/da-workstation/tasks/`.
In a task: Enter sends a follow-up on the same Muse session, Ctrl+. stops the run (interrupt, then its process group), Esc goes back.
Layout: status pills pair an icon with text; long titles end in "…"; the input scrolls sideways so the cursor stays visible.
The home view is a Slack-style list (D-011). Agent sessions: one row per task with a status icon, bold when it has new agent messages, an unread count, and "⚠ you" while a request waits for you. Apps: one row per app with initials, window count, an agent tag for terminals that name Claude, Codex, Muse or Gemini, and a green dot when focused. Clicking an app opens its chat: its windows with Focus buttons and a timeline of opened, closed, renamed and focused windows since the sidebar started.
A command with several stages (`a && b | c`) needs one answer: Muse asks per stage, and the sidebar repeats your choice for the rest of that command.
Alerts: one per real transition (done, failed, needs permission), in the sidebar and as a desktop notification; clicking it opens the task.
A restart marks an active run "Interrupted" until you check it.
Muse runs through `muse serve` in the task folder (D-010). When Muse asks to run a command or touch a file, the task shows "⚠ Needs permission" with "Allow once" and "Reject"; the request stays until Muse confirms the decision. Muse's own approval judge is switched off so you decide.
The chat shows Muse's messages, each tool call as an event line (such as `bash date > probe.txt: completed`), the outcome, and token usage. The task's first event line names the Muse session; `muse resume <id>` in that folder opens it.
A follow-up resumes the same Muse session, so Muse keeps its context. `DA_MUSE_TRANSPORT=exec` switches to `muse exec --json` (no approvals).
November 2: Muse Code 1.4.14 with a subscription login runs real tasks.
When things fail: no Muse binary gives an install alert; no Muse login fails the run with a sign-in step (run `muse`, type /login, then send a follow-up); a missing @folder gives an alert. If the sidebar dies, `muse serve` exits as soon as its input closes, so no run continues unseen; the task shows Interrupted on restart.

## Tiling ideas

November 1 review of existing GNOME tiling extensions. These are proposals, not decisions.

| Project | License | GNOME 50 | Idea to borrow |
|---|---|---|---|
| Tiling Shell (domferr) | GPL-3.0 | yes (42–51) | JSON layouts, auto-tile new windows, tiles that span |
| Tiling Assistant (leleat) | GPL-2.0+ | yes (48–51) | tile groups raised together, untile restores size |
| PaperWM | GPL-3.0 | packaged as 50.x | scrolling columns; keyboard-first moves |
| Pop Shell, Forge | GPL-3.0 | no; archived or ending | directional focus and move |

Copying their code puts GPL terms on `gnome-extension/`. The Rust app talks to it over D-Bus, a separate process.
Agent proposal: keep our small adapter. Next steps are any window count (master plus a split stack), directional focus, and a restore snapshot that survives extension reload.
