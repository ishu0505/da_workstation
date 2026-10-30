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

## Boundaries

- October 28–29, 2026 are planning only. No implementation is scheduled October 30.
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

## Check

Application checks are not established. The command below checks kit documents only.

```sh
sh .agents/kit/check.sh
```
