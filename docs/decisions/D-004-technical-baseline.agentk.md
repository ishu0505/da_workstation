---
id: D-004
status: proposed
date: 2026-10-29
supersedes: none
source_paths: []
---

# D-004: Propose one native Rust application with GPUI and SQLite

- **Decision:** Propose macOS, Rust 2024, GPUI, and SQLite in one application. Use logical UI, task, provider, platform, and store modules with typed calls/events. Keep blocking provider I/O off the UI thread. Propose one active editing run per directory, a 70/30 layout, and no separate daemon initially. Do not create application code or dependencies during setup.
- **Why:** The handoff proposes a small native application for the available development machine and the short build window. Technical suitability is not verified.
- **Rejected:** Linux/GNOME is deferred in the handoff. A daemon or workflow engine needs a demonstrated requirement. Framework comparisons: not recorded.
- **Evidence:** PROJECT_HANDOFF.agentk.md sections 7–10. No dependency builds, API experiments, benchmarks, or platform checks have run.
- **Reverse if:** Failure of a required capability or dependency compatibility check prompts a user decision before replacing the proposal.
- **Open:** Confirm demo platform and layout. Verify GPUI version, Xcode/Metal prerequisites, async bridge, window permissions, Muse transport, and dependencies October 31. Budget allocations are proposals; provider attribution and remaining credits are unverified.
