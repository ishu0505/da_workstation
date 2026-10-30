---
id: D-002
status: accepted
date: 2026-10-29
supersedes: none
source_paths: []
---

# D-002: Implement Muse first behind a provider boundary

- **Decision:** Implement one Muse adapter first. Tasks select a provider explicitly or inherit a default. Preserve independent provider/session ownership for future Codex/ChatGPT and Claude adapters, per-task selection, and mixed-provider execution. Normalize capabilities, start, continuation, events, approvals, usage, and cancellation. Use a portable handoff across providers; do not promise native session transfer.
- **Why:** The user requires real Muse delegation first while preserving future provider expansion.
- **Rejected:** Implementing additional providers before the required build is deferred. Automatic routing and fallback rules are future work.
- **Evidence:** PROJECT_HANDOFF.agentk.md sections 4 and 7; user setup instructions. No protocol or entitlement verification yet.
- **Reverse if:** An explicit user requirement change or verified interface constraint warrants a new decision; report constraints before changing required scope.
- **Open:** Muse build, protocol, authentication, and capability details require verification October 31. Future subscription tooling and entitlements require separate verification.
