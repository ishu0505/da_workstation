---
id: D-003
status: accepted
date: 2026-10-29
supersedes: none
source_paths: []
---

# D-003: Keep persistent sidebar chats and execution evidence

- **Decision:** Each task has a persistent sidebar chat containing progress, results, alerts, and follow-up instructions. Preserve goal, project, provider/session reference, conversations, run attempts, artifacts, and next action. Support blocked, failed, cancelled, and completed outcomes. Reconcile old runs after restart before reporting completion. Distinguish an agent final message from verified success.
- **Why:** The user needs to leave running work, receive useful updates, and return without losing context.
- **Rejected:** A mock result or screenshot-only prototype does not satisfy the required flow. Exact rejected storage designs: not recorded.
- **Evidence:** PROJECT_HANDOFF.agentk.md sections 2, 5, and 6; user setup instructions. Persistence and recovery are not implemented.
- **Reverse if:** An explicit user requirement change can supersede this record. Implementation constraints must be reported with alternatives.
- **Open:** Exact schema, state names, event ordering, recovery protocol, and context-card presentation remain implementation design work.
