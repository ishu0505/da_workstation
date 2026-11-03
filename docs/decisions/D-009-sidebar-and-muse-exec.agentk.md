---
id: D-009
status: accepted
date: 2026-11-02
supersedes: []
source_paths: [src/sidebar.rs, src/tasks.rs, src/provider.rs, src/provider/muse.rs, src/notify.rs, gnome-extension/extension.js]
---

# D-009: Sidebar window with task chats; Muse through `muse exec` first

- **Decision:** Add the sidebar as a second GPUI window (`--sidebar`, app id `dev.da.Workstation.Sidebar`) that the GNOME extension docks on the left; tiling leaves its width free. Store each task as one JSON file. Run Muse through the documented headless `muse exec "<prompt>"` behind the provider boundary; a follow-up is a new run with a portable handoff (goal plus recent chat). Show Claude/Codex terminals only as tagged open-app rows. Supplements D-002, D-003 and D-008.
- **Why:** The user asks for a Slack-like left panel with agent sessions, open apps, notifications and a Muse chat, on November 2. The user chose to show other agents as app rows. Agent inference: Wayland apps cannot place their own windows, so the extension docks the sidebar; JSON files need no new crate in an offline build; `muse exec` is the only Muse interface documented on dev.meta.ai/docs/muse-code that needs no schema inspection.
- **Rejected:** Agent inference: SQLite (D-004 proposal) needs a new crate and network for the build; a TypeScript SDK bridge adds a Node runtime; waiting for `muse serve` blocks the sidebar on the Muse install.
- **Evidence:** 18 Rust tests pass, including a stand-in `muse` that streams output, reports exit status and is stopped with its process group. The headless GNOME harness opens the sidebar, docks it at x = 0 with width 380, and tiles three windows to its right.
- **Reverse if:** Agent proposal: the installed Muse exposes approvals only through `muse serve`; then the adapter moves to that protocol without a task schema change.
- **Update (November 2, later):** Muse Code 1.4.14 is installed with an API key. A real `muse exec` run reached Meta's API and failed with 402 "Billing verification failed" (account billing, not code). The adapter now runs `muse exec --json --session-id <uuid>`, turns `run.output.delta` events into chat lines and takes the outcome and reason from `run.terminal.*`; checked end-to-end against the real binary with its offline `echo` provider (20 tests plus 1 ignored real-binary test). `muse schema` lists `approval/requested`, `approval/decide`, `session/resume`, `turn/start` and `turn/interrupt` on `muse serve`, so the reverse condition holds for approvals: the move to `muse serve` is the next step. Whether a reused `--session-id` carries model context is unverified, so follow-ups still send the portable handoff.
- **Update (November 2, subscription):** With a Muse subscription login, real runs work: one run wrote `hello.txt` (Muse's approval judge allowed it; no approval event reached the client). Reusing `--session-id` keeps the model's context (the second run recalled a code word), so a follow-up now sends only the new instruction on the same session; the portable handoff stays for sessions a provider cannot resume. Tool results show in the chat as event lines. Evidence: 20 tests plus 2 ignored real-binary tests (`cargo test -- --ignored`), one of them two real `meta` runs.
- **Update (November 2, D-010):** The Muse transport moved to `muse serve`; `muse exec` remains as a fallback.
- **Open:** Muse Code is not installed on this machine; no real Muse run is verified. Approvals, session resume and usage need `muse serve` (`muse schema`). Notification click relies on the desktop's `ActionInvoked` signal; not yet seen on the user's desktop.
