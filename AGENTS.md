# DA Workstation

A native workstation for window focus, tiling, and persistent Muse task chats.

Priorities: required working flow, reliable task state, clear UX, cost, expansion.

## Read when needed

- `PROJECT_HANDOFF.agentk.md`: detailed requirements, acceptance, and open choices.
  Read before changing scope, milestones, or provider boundaries.
- `PLAN.agentk.md`: concise work tracker.
- `DECISIONS.agentk.md`: decisions, reasons, and proposal status.
- `KIT.agentk.md`: kit commands.
- `experiments/EXPERIMENTS.agentk.md`: experiment evidence.
- `SUBMISSION.agentk.md`: hackathon package draft (description, setup, demo, limits, checklist).

## Boundaries

- October 28–29, 2026 are planning only. October 30 starts the launcher (D-006) and existing-window focus (D-007).
- Implement October 31–November 2; finish November 2; submit November 3.
- All dates use Asia/Singapore. Submission requires explicit user authorization.
- Preserve native tiling, the focus/command bar, and real Muse delegation.
- Each task keeps its sidebar chat, progress, results, alerts, and follow-ups.
- Implement Muse first. Preserve per-task provider selection and future
  mixed-provider execution.
- Add Slack capture, reminders, and advanced automation after the required flow.
- Keep provider-specific behavior behind the provider boundary.
- Treat unverified technical choices as proposals.
- Do not commit or push without authorization.

## Working rules

- Keep this map short; link to detail.
- Preserve the detailed handoff.
- Write kit documents with the .agentk.md suffix.
- Leave other Markdown files unchanged; report drift.
- Add application commands only after verifying them.
- When /graphify is invoked, read ~/.agents/skills/graphify/SKILL.md first.
- Read folder maps before working inside those folders.
- Run the wrap-up skill after tasks that change files.
- Kit skills are in `.agents/skills/`; settings are in `.agents/agent-kit.conf`.

## Tasks

- Open a task file (`sh .agents/kit/task.sh new <name>`) when the work spans
  sessions or units, needs my OK on a plan, or is an investigation to keep.
  Tell me; I can say no. Small fixes, questions, doc fixes: no task file.
- After each step, update its Found, Decisions, Done and Next. Continue a
  task only from its file.

## Check

Application entry point: `src/main.rs`; scope and usage: `FOCUS_BAR.agentk.md`.
Window focus and first tiling/restore controls: `src/windows.rs` and `gnome-extension/`; setup: `gnome-extension/README.agentk.md`.
The sidebar is the whole app (D-012, D-013); Muse's desktop tools: `src/mcp.rs`; Help page: `src/help.rs`; task store and Muse adapter (`muse serve`, D-010): `src/sidebar.rs`, `src/switcher.rs`, `src/activity.rs`, `src/tasks.rs`, `src/provider/`.
Isolated GNOME evidence: `experiments/gnome-focus/REPORT.agentk.md`.
The command below checks kit documents only.

```sh
sh .agents/kit/check.sh
```
