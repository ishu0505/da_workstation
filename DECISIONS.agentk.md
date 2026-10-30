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
| [D-001](docs/decisions/D-001-scope-and-schedule.agentk.md) | Preserve the required flow and three implementation days | accepted | 2026-10-29 |
| [D-002](docs/decisions/D-002-provider-boundary.agentk.md) | Implement Muse first behind a provider boundary | accepted | 2026-10-29 |
| [D-003](docs/decisions/D-003-task-continuity.agentk.md) | Keep persistent sidebar chats and execution evidence | accepted | 2026-10-29 |
| [D-004](docs/decisions/D-004-technical-baseline.agentk.md) | Propose one native Rust application with GPUI and SQLite | proposed | 2026-10-29 |
| [D-005](docs/decisions/D-005-context-kit.agentk.md) | Install personal context maps and planning documents only | accepted | 2026-10-29 |
