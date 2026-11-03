# DA Workstation: submission package

Draft for the user's review, November 2, 2026 (Asia/Singapore). Nothing here is submitted.
Submission needs the user's explicit OK (November 3). The organizer's deadline is
November 4, 9 PM ET (November 5, 10 AM Singapore). Recheck it on the portal.

## 1. Project description

**Short (one line).** A native Linux workstation that tiles your windows, finds any app
from one sidebar, and runs Meta's Muse Code agent in persistent task chats, where you
approve each command before it runs.

**Long.** Developers juggle windows, terminals and coding agents. DA Workstation puts
them in one keyboard-first surface on GNOME:

- One app, docked on the left, with one typing box: search sessions and apps, launch an app,
  or ask Muse (Ctrl+Alt+Space from anywhere).
- Native tiling puts one big window beside the panel and the others as small live cards on the
  right, each captioned with the app and what it shows (a browser's page title). New windows tile
  themselves, from any launcher; Ctrl+Super+R restores. Click a card to swap it in.
- The left panel is for conversations, like the Claude and ChatGPT apps: New task, what needs
  you, sessions by day. Type `/` for commands (/tile, /restore, /bottom, /autotile off). The ? button
  opens a guide in the browser with every shortcut, command and Muse tool.
- The panel docks on the left or as a bottom bar (one click to switch).
- Muse can drive the desktop: ask in plain language ("open Chrome and tile it big next to my
  terminal") and it uses the workstation's own tools (open apps, focus, tile, open a URL) without
  asking each time. Shell commands and file edits still need your approval.
- The sidebar works like Slack: a "Needs you" inbox (approve without opening the chat), agent
  sessions sorted by urgency, and apps, each with its own chat. An app's chat
  lists its windows (Focus) and what happened to them. Each task runs Muse Code in a chosen
  project folder through Muse's session protocol (`muse serve`). Muse's messages, tool steps, token
  usage and changed files appear live. When Muse wants to run a command or edit a file,
  the task shows "⚠ Needs permission" with **Allow once** and **Reject**. You can stop a run
  and send follow-ups on the same Muse session.
- A context card at the top of each task shows the goal, changed files, outcome and the
  next action. Tasks survive a restart. A run that was active is marked "Interrupted",
  never reported as done.

## 2. Track and theme (user to choose)

- Proposed track: **Indie developer** (one builder).
- Proposed theme: **Build for Your World**. It brings Meta's Muse Code into the
  developer's own daily workflow.
- Meta technology used: **Muse Code 1.4.14** (the coding agent), driven through `muse serve`.

## 3. Architecture summary

One Rust 2024 application (GPUI 0.2.2) and one small GNOME Shell extension.

```
Sidebar (GPUI): search, sessions, apps, chats
 ├── D-Bus (session bus) ── GNOME extension (GJS, in gnome-shell):
 │                          focus, card-deck tiling, restore, outline, docking
 ├── Task store: one JSON file per task (~/.local/share/da-workstation/tasks)
 └── Provider boundary (trait) ── Muse adapter ── `muse serve` (JSON-RPC over stdio)
```

- `src/main.rs`: the CLI commands (and the older floating bar, no longer bound to a shortcut).
- `src/switcher.rs`: the typing box's search and choices.
- `src/mcp.rs`: the workstation's tools for Muse (`--mcp`).
- `src/sidebar.rs`: task chats, open apps, alerts and the context card.
- `src/tasks.rs`: the task model and store.
- `src/provider/`: the provider trait, the Muse adapter (`muse serve`), and the `muse exec`
  fallback.
- `gnome-extension/`: the D-Bus interface. Wayland apps cannot place windows themselves,
  so the extension does.
- Decisions: D-006 to D-010 in `DECISIONS.agentk.md`. D-010 records `muse serve` and approvals.

## 4. Build and setup

Verified on one machine: Fedora 44, GNOME 50 on Wayland, rustc 1.99.1, Muse Code 1.4.14.
A clean-machine install has not been tried.

1. Install the build packages: Rust and Cargo, plus the GPUI Wayland development headers.
   This machine has `rust`, `cargo`, `wayland-devel`, `libxkbcommon-devel` and `gtk4-devel`.
2. Build: `cargo build --locked` (gives `target/debug/da-workstation`).
3. Install the GNOME extension (it needs `layout.js` packed with it):
   ```sh
   gnome-extensions pack --force --extra-source=layout.js --out-dir /tmp/da-workstation-extension gnome-extension
   gnome-extensions install --force /tmp/da-workstation-extension/da-workstation@local.shell-extension.zip
   gnome-extensions enable da-workstation@local
   ```
   Then sign out and back in. GNOME loads extension code only at sign-in.
4. Install the shortcuts: `python3 scripts/install-shortcut.py`. This sets Ctrl+Alt+Space
   for the bar, Ctrl+Alt+B for the sidebar, and Ctrl+Super+arrows to move focus.
5. Install Muse Code and sign in once: run `muse` in a terminal and type `/login`.
6. Check: `cargo test --locked` runs 27 tests. The isolated GNOME check is
   `python3 -I experiments/gnome-focus/verify.py` (15 checks).

## 5. Demo script (about 3 minutes)

Before recording:
1. Run `sh demo/prepare.sh`. It copies the fixture project, which has one planted bug and
   two failing tests, to `~/da-demo-pricing`.
2. Open two or three normal windows (for example a terminal, a browser and an editor).
3. Close old task chats if you want a clean list.

| Step | Do | Viewers see |
|---|---|---|
| 1 | Ctrl+Alt+Space | The sidebar docks on the left with the typing box focused |
| 2 | Type "term", Enter | The open terminal is focused (no second window) |
| 3 | Type "files", Enter | Files opens as the big window; the other windows become cards (no tiling key) |
| 3a | Click "⇩ Bottom bar" in the header | The panel moves to the bottom; the windows re-tile above it. Click again to return |
| 3b | Click a card | It swaps with the big window; no window is resized |
| 3c | Ctrl+Alt+Space, type "open the Meta Muse docs in the browser and make it the big window", Enter | Muse opens the page and tiles it with its own tools; no permission prompt |
| 4 | Ctrl+Super+Left/Right | Focus moves between tiled windows |
| 5 | In the sidebar: `@~/da-demo-pricing run the tests, find the bug, fix it, and run the tests again`, Enter | Task "Running"; Muse messages and tool steps stream in |
| 6 | Click another window while Muse works | Muse keeps working; nothing types into your apps |
| 7 | First "⚠ Needs permission" (a shell command, such as `python3 -m unittest -v`) → **Reject** | The rejection is recorded; Muse says what it could not do |
| 8 | Follow-up: "you may run the tests now", Enter → **Allow once** | The tests run; two failures are shown as evidence |
| 9 | Approve the edit, then the test rerun | The card shows Changed: `pricing.py`; the outcome is "Done"; the end line lists the files and token usage |
| 10 | In the terminal: `cd ~/da-demo-pricing && python3 -m unittest` | You check the result yourself: OK |
| 11 | Ctrl+Super+R | The windows return to their earlier positions; the cards go away |
| 12 | Ctrl+Alt+Space, type part of the task title, Enter | The task opens with its history and card |

The dry run with real Muse on November 2 fixed the bug in 38 s (about 32,000 tokens). That
run left Muse's own auto-approver on, so it asked no questions. In the app it is off, so
expect three or four permission requests.

## 6. Evidence (November 2)

| Claim | Evidence |
|---|---|
| Tiling, focus, restore, docking, glow, expand | Isolated GNOME 50 run: 18 PASS (`experiments/gnome-focus/verify.py`); user desktop check, November 2 |
| Real Muse delegation with approvals | Real adapter tests pass 4/4: echo run, follow-up keeps context, reject then allow, stop |
| Stop | Muse cancels the turn in about 13 ms; the process group ends after 5 s if needed |
| No login | The run fails with Muse's `authRequired` and a sign-in step (recorded message, unit test) |
| App crash during a run | `muse serve` exits 0.1 s after its input closes; the command does not finish; restart marks the task Interrupted |
| Persistence | Store tests: atomic JSON writes; restart marks active runs interrupted |
| Stage | GNOME check: clicking or focusing a card makes it the big window; no window is resized |
| Auto-tiling, bottom bar | GNOME checks: an app opened through DA Workstation is the big window; bottom-bar mode tiles above the bar |
| Muse desktop tools | Real test: Muse listed the user's windows through the workstation tool with no permission prompt |
| One answer per command | Real test: a three-stage shell command needs one Allow and finishes |
| Unit tests | `cargo test`: 33 pass |

Details: `experiments/EXPERIMENTS.agentk.md`, `experiments/gnome-focus/REPORT.agentk.md`, D-010.

## 7. Known limits

- Linux GNOME 50 on Wayland only. The original macOS target was moved to GNOME (D-007).
  Other desktops are not supported.
- Extension changes need a sign-out/in. The sidebar docks only with the extension running.
- Tiling: current monitor and workspace, one stage layout (big window plus cards). Cards are
  live but small; type into a window after clicking it.
- Muse's browser use is "open a URL" only; it cannot read or click inside web pages. It refuses fullscreen and
  non-resizable windows. Large minimum window sizes can limit the layout.
- One provider (Muse). The provider boundary allows more, but none is built. Concurrent
  runs in several tasks are not verified.
- Approvals depend on `MUSE_DISABLE_APPROVAL_JUDGE=1`, a setting found in the Muse binary,
  not in its docs (D-010). Without it, Muse's judge approves most actions itself.
  `DA_MUSE_TRANSPORT=exec` is the fallback, without approvals.
- Changed files come from Muse's file tools only. Files changed by shell commands are not listed.
- Usage is in tokens. Muse reports no money cost, so none is shown.
- Storage is JSON files, not SQLite (the planned database was not needed for one user).
- The bar reads tasks when it opens; it does not update live.
- Not checked: network failure during a run, a clean-machine install, multiple monitors.
- Not built: Slack capture, reminders, other agents.

## 8. Submission checklist

| # | Item | Owner | State |
|---|---|---|---|
| 1 | Desktop check: tiling, focus, docked sidebar, a real task with an approval | user | done (November 2) |
| 2 | Rehearse the demo three times on the fixture; note any step that differs | user | open |
| 3 | Record the demo video (check the portal's length and format rules first) | user | open |
| 4 | Read the official rules and the portal fields (not yet published on October 28 or November 2) | user + agent | open (November 3) |
| 5 | Choose track and theme (proposal in section 2) | user | open |
| 6 | Repository: commit the work, push, and make the repo visible if the portal needs a link (`origin` is `github.com/ishu0505/da_workstation`) | user authorizes; agent can do it | open; nothing is committed yet |
| 7 | `README.md` for judges: the user's file, now only a title. Agent may write it from sections 1, 4 and 5 if the user agrees | user | open |
| 8 | License check: `LICENSE-GPUI` is in the repo; choose this project's license | user | open |
| 9 | Fill the portal with sections 1–3 and the links | user | open |
| 10 | Submit, then record the confirmation | user (explicit OK) | open |
