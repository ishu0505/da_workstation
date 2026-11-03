# Plan

## Goal

Build a native developer workstation with tiling, a focus/command bar, and real Muse delegation.
Detailed requirements and acceptance gates: [project handoff](PROJECT_HANDOFF.agentk.md), sections 2–9.
Decisions and proposal status: [decision index](DECISIONS.agentk.md).
All dates below are in 2026, Asia/Singapore.

## First milestone

Finish the required build November 2. Prepare submission for November 3; submission needs explicit user authorization.
Slack capture, reminders, and advanced automation follow the required working flow.

## Work tracker

- [x] October 28–29: preserve the handoff; approve and install the context kit; record planning decisions.
- [ ] October 29: resolve remaining UX choices and refine contracts and acceptance details, without implementation.
- [x] October 30: clarify permission alerts per task and future agent capability management in the handoff, section 6.
- [x] October 30: build and verify a Rust/GPUI focus bar on the current Fedora machine; search and launch apps, including terminals (D-006).
- [x] October 30: implement existing-window focus through a GNOME extension; pass isolated GNOME 50 integration checks (D-007).
- [x] October 30: enable the installed extension on the user's desktop; verify the Rust focus command reaches Ghostty without a duplicate window.
- [x] October 30: verify typing, selection, and Ghostty focus through the bar on the user's desktop; user confirms no duplicate window.
- [x] October 30: verify Ctrl+Alt+Space summons the bar; user confirms success after re-registering the shortcut.
- [x] October 30: verify the summon shortcut reopens the bar after Escape; user confirms success.
- [x] October 30: implement first GNOME tiling/restore controls; isolated two-window geometry and restoration checks pass.
- [x] November 1: user confirms Ctrl+T/Ctrl+R on the desktop after sign-out/in. Minimum-size constraints remain unchecked.
- [x] November 2: tile any count, Ctrl+2–9 recent windows, Ctrl+Super+arrows focus, restore across reload; 14 headless checks pass. Desktop check needs sign-out/in.
- [x] November 2: sidebar window with task chats, open apps, alerts; Muse adapter through `muse exec` (D-009). 18 Rust tests pass.
- [x] November 2: Muse Code 1.4.14 installed; adapter reads `muse exec --json` events (D-009 update).
- [x] November 2: real Muse runs work with a subscription login; follow-ups continue the Muse session (D-009 update).
- [x] November 2: Muse through `muse serve`: real approval (reject and allow), resume, stop, usage (D-010).
- [x] November 2: task card (goal, changed files, outcome, next action); files changed per run; sidebar keeps the last folder; bar lists and opens tasks; sidebar re-docks after login. 27 Rust tests pass.
- [x] November 2: sidebar restyle (status pills, scrolling input, bubbles) and glow frames on tiled windows; 15 headless checks pass. Glow needs sign-out/in.
- [x] November 2: submission package draft (`SUBMISSION.agentk.md`), demo fixture (`demo/`); real Muse dry run fixes it in 38 s.
- [x] November 2: one answer per multi-stage command; focus columns with hover-to-expand; Slack-style sidebar with app chats (D-011). 18 GNOME checks, 30 tests.
- [x] November 2: card deck tiling; the sidebar is the whole app (search, tiling keys, Needs-you inbox, urgency order, Slack-style chat) (D-012). 18 GNOME checks, 31 tests.
- [x] November 2: stage tiling (big window + scaled cards), Ctrl+Super+T/R, Muse desktop tools without prompts, one typing box (D-013). 18 GNOME checks, 33 tests, real tool test.
- [x] November 2: apps opened from DA Workstation tile themselves; bottom-bar placement with a header switch (D-014). 20 GNOME checks.
- [x] November 2: chat-style left panel, captioned cards on the right, auto-tile every window, `/` commands (D-015). 21 GNOME checks, 32 tests.
- [x] November 2: Help guide opened in the browser (? button, /help, Ctrl+/), generated from the command and Muse tool lists. 33 tests.
- [ ] November 2: user runs one real Muse task with an approval from the sidebar on the desktop.
- [ ] October 31: prove the native shell, window focus/tiling, and one real Muse task. Record evidence.
- [ ] November 1: integrate persistent sidebar chats, task state, approvals, cancellation, alerts, and follow-up instructions.
- [ ] November 1: verify Muse permission requests in the matching sidebar tab. Show a clear label, icon, and highlight. Test approval and denial.
- [ ] November 1: pass the required demo end to end; freeze features and list defects.
- [ ] November 2: verify failures and recovery; fix defects; package the build and rehearse the demo.
- [ ] November 2: finish build, evidence, demo material, limitations, and submission checklist.
- [ ] November 3: recheck portal requirements; present the exact package; submit only after authorization.

## Planned units

### PRD v4 coverage

The October 30 blueprint is captured in [handoff section 12](PROJECT_HANDOFF.agentk.md#12-prd-v40-vision-and-coverage-october-30).
It extends the product vision; it does not replace the November 2 acceptance gates or implementation dates.

- [x] Capture deterministic control plus autonomous agents, local AI routing, and the Active Context Sentinel / Process Watcher.
- [ ] October 31: verify the available Muse interface and target platform before choosing concrete integrations.
- [ ] Required build: preserve task context, real run monitoring, permission alerts, cancellation, and follow-up in each task chat.
- [ ] After the required flow: assess independent process watchers, context restoration across profiles, and bounded browser automation.
- [ ] Later roadmap: local inference, subscription adapters, automatic routing, full command DSL/frecency, Linux support, and vision fallback.
- [ ] Validate v4 latency/cost claims, background input isolation, model capabilities, and platform permissions before promising them.

### Initial application

One application initially. Proposed logical modules: UI, task service, Muse adapter, platform service, and store.
The first application lives in `src/` with `Cargo.toml` at the root.
It starts with UI, Linux desktop discovery/launch, and GNOME window focus; see [focus bar](FOCUS_BAR.agentk.md).
The small GNOME companion and isolated focus experiment use root map pointers; both remain below unit-map thresholds.
Add task/provider/store modules as the required flow needs them. Add unit maps when kit thresholds require them.
See handoff section 7 and D-004 for the proposed technical baseline.

## Open questions

- Confirm macOS, demo apps/display geometry, summon shortcut, and layout ratio.
- Verify installed Muse protocol/authentication and choose a bounded demo project on October 31.
- Validate Rust/GPUI/SQLite and the async bridge; pin dependencies only during implementation.
- Additional development agents and CI checks remain undecided; current setup uses Codex and local checks.
- Slack authorization is needed only if live capture is attempted after the required flow works.
- Proposed sidebar colors need contrast checks. Labels and icons must explain each status without color.
- Plan skill, MCP connection, plugin, and tool management after the required flow. Verify the controls each agent exposes first.
- Keep permission requests separate when concurrent runs ship. Test that each approval reaches only its matching task and run.
- See handoff sections 9–10 for unverified budget details and remaining choices. GNOME focus evidence is in [the experiment report](experiments/gnome-focus/REPORT.agentk.md).
