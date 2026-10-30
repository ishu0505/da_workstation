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
- [ ] October 30: no implementation scheduled; optional planning only.
- [ ] October 31: prove the native shell, window focus/tiling, and one real Muse task. Record evidence.
- [ ] November 1: integrate persistent sidebar chats, task state, approvals, cancellation, alerts, and follow-up instructions.
- [ ] November 1: pass the required demo end to end; freeze features and list defects.
- [ ] November 2: verify failures and recovery; fix defects; package the build and rehearse the demo.
- [ ] November 2: finish build, evidence, demo material, limitations, and submission checklist.
- [ ] November 3: recheck portal requirements; present the exact package; submit only after authorization.

## Planned units

One application initially. Proposed logical modules: UI, task service, Muse adapter, platform service, and store.
They use typed calls and events; no source folders or application manifests exist yet.
Create folders and unit maps only when implementation starts and kit thresholds require them.
See handoff section 7 and D-004 for the proposed technical baseline.

## Open questions

- Confirm macOS, demo apps/display geometry, summon shortcut, and layout ratio.
- Verify installed Muse protocol/authentication and choose a bounded demo project on October 31.
- Validate Rust/GPUI/SQLite and the async bridge; pin dependencies only during implementation.
- Additional development agents and CI checks remain undecided; current setup uses Codex and local checks.
- Slack authorization is needed only if live capture is attempted after the required flow works.
- See handoff sections 9–10 for unverified budget details and remaining choices. No experiments have run.
