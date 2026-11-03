---
id: D-010
status: accepted
date: 2026-11-02
supersedes: []
source_paths: [src/provider/muse_serve.rs, src/provider/muse.rs, src/provider.rs, src/sidebar.rs, src/tasks.rs]
---

# D-010: Muse through `muse serve` so the user answers approvals

- **Decision:** The Muse adapter talks the Muse Session Protocol (MSP v1, JSON-RPC 2.0, one JSON object per line) to `muse serve`, one process per run: `initialize`, `initialized`, `session/start` (approval mode `promptUnmatched`) or `session/resume` plus `session/setApprovalMode`, then `turn/start`. Each `approval/request` becomes a sidebar request with "Allow once" and "Reject"; the choice goes back as `approval/decide`, and the request stays shown until `approval/resolved` arrives. Stop sends `turn/interrupt`, then ends the process group after 5 s. `muse exec --json` stays as a fallback (`DA_MUSE_TRANSPORT=exec`). Replaces the `muse exec` path of D-009; the rest of D-009 stands.
- **Why:** The required flow (handoff section 5) needs the user to answer a real approval request. `muse exec` cannot. The user asked to continue toward the goal on November 2.
- **Rejected:** Agent inference: keeping Muse's built-in LLM approval judge on: it allowed a `write_file` and a `date > probe.txt` shell call by itself, so no request reached the user. Agent proposal: the adapter sets `MUSE_DISABLE_APPROVAL_JUDGE=1`, a switch found in the 1.4.14 binary and not in the public docs; reverse if Muse removes it or documents another way.
- **Evidence:** A Python probe and four ignored Rust tests against Muse Code 1.4.14 with a subscription login (`cargo test -- --ignored`): a real shell approval arrives; "Reject" leaves no file and Muse records `abort (by user)`; "Allow once" creates the file; a follow-up on a resumed session recalls an earlier code word; Stop ends a running turn as "cancelled" within 13 ms. 23 unit tests read recorded message shapes. Usage per run comes from `session/tokenUsage` (about 57k tokens for one small turn).
- **Reverse if:** A newer Muse changes the MSP stable surface (fingerprint `sha256:c94c3d1f…` in `muse schema`), or the judge switch stops working; then fall back to `DA_MUSE_TRANSPORT=exec` and record the change.
- **Open:** Approval choices other than one-time (`session`, `localPersistent`) are not offered. `userInput/request` prompts are not shown yet. Not yet seen in the sidebar on the user's desktop.
