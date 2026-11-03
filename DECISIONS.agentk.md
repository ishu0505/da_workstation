# Decisions

The index of every decision: what was decided, why, and on what evidence.
Each decision is one file in `docs/decisions/`, named
`D-NNN-<short-name>.agentk.md` (`DECISIONS_DIR` in
`.agents/agent-kit.conf`), written with the `lineage` skill. A file is never
rewritten; a changed mind is a new decision, and the old one's status
becomes `superseded by D-NNN`. `check.sh` keeps this table and the files in
step.

| ID | Decision | Status | Date |
|---|---|---|---|
| [D-001](docs/decisions/D-001-scope-and-schedule.agentk.md) | Preserve the required flow and three implementation days | superseded by D-006 | 2026-10-29 |
| [D-002](docs/decisions/D-002-provider-boundary.agentk.md) | Implement Muse first behind a provider boundary | accepted | 2026-10-29 |
| [D-003](docs/decisions/D-003-task-continuity.agentk.md) | Keep persistent sidebar chats and execution evidence | accepted | 2026-10-29 |
| [D-004](docs/decisions/D-004-technical-baseline.agentk.md) | Propose one native Rust application with GPUI and SQLite | proposed | 2026-10-29 |
| [D-005](docs/decisions/D-005-context-kit.agentk.md) | Install personal context maps and planning documents only | accepted | 2026-10-29 |
| [D-006](docs/decisions/D-006-first-launcher.agentk.md) | Start the Rust app launcher October 30; retain the required flow and delivery dates | accepted | 2026-10-30 |
| [D-007](docs/decisions/D-007-gnome-focus.agentk.md) | Add existing-window focus through a narrow GNOME Shell extension | accepted | 2026-10-30 |
| [D-008](docs/decisions/D-008-gnome-tiling.agentk.md) | Add native GNOME master/stack tiling and restoration controls | accepted | 2026-10-30 |
| [D-009](docs/decisions/D-009-sidebar-and-muse-exec.agentk.md) | Sidebar window with task chats; Muse through `muse exec` first | accepted | 2026-11-02 |
| [D-010](docs/decisions/D-010-muse-serve-approvals.agentk.md) | Muse through `muse serve` so the user answers approvals | accepted | 2026-11-02 |
| [D-011](docs/decisions/D-011-focus-columns-and-slack-sidebar.agentk.md) | Focus columns with hover-to-expand; Slack-style sidebar | superseded | 2026-11-02 |
| [D-012](docs/decisions/D-012-card-deck-and-sidebar-app.agentk.md) | Card deck tiling; the sidebar is the whole app | superseded in part by D-013 | 2026-11-02 |
| [D-013](docs/decisions/D-013-stage-tools-one-box.agentk.md) | Stage tiling, desktop tools for Muse, one typing box | accepted | 2026-11-02 |
| [D-014](docs/decisions/D-014-auto-stage-and-bottom-bar.agentk.md) | Apps opened from DA Workstation tile themselves; bottom-bar placement | accepted | 2026-11-02 |
| [D-015](docs/decisions/D-015-chat-panel-and-auto-tile.agentk.md) | Chat-style left panel, apps as cards on the right, auto-tile every window | accepted | 2026-11-02 |
