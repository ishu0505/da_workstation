---
id: D-005
status: accepted
date: 2026-10-29
supersedes: none
source_paths: []
---

# D-005: Install personal context maps and planning documents only

- **Decision:** Install kit 0.7.0 with personal ownership, share mode agent, Codex as the development agent, no pointer files, and no unit folders. Use .agentk.md documents, separate decision records, an empty experiment ledger, local hooks, and no CI setup. Defer learning docs. Preserve the handoff and graphify invocation rule. Stage shared setup files; do not commit or push.
- **Why:** The user requested context-kit setup and planning only, a short root map, a concise tracker, and preserved detailed requirements. The user approved the concrete proposal and defaults.
- **Rejected:** Team beside mode does not match the user-confirmed personal repository. Additional development-agent wiring, CI, and learning docs are deferred.
- **Evidence:** User approval of the setup proposal on October 29, 2026. Source templates: agent_dev_kit 0.7.0. No cleanup was needed.
- **Reverse if:** An explicit change in repository ownership, sharing preference, or development agents warrants a new decision and kit configuration update.
- **Open:** Additional development agents and CI preferences remain open. Hook files can be inspected, but live agent-hook invocation is unverified.
