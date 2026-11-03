---
id: D-007
status: accepted
date: 2026-10-30
supersedes: []
source_paths: [src/windows.rs, src/main.rs, gnome-extension/extension.js, gnome-extension/metadata.json]
---

# D-007: Add existing-window focus on GNOME

- **Decision:** Extend the October 30 launcher with existing-window focus through a narrow GNOME 50 Shell extension. This supplements D-006.
- **Why:** The user approves focus as the next increment. Agent inference: focus moves the launcher toward the required workstation flow.
- **Rejected:** Direct GNOME `FocusApp` and introspection calls return `AccessDenied` to ordinary callers. The built-in `FocusApp` also targets the app grid. Agent inference: a narrow extension avoids arbitrary Shell evaluation and keeps compositor behavior behind the platform boundary. Wider alternatives remain open.
- **Evidence:** `experiments/gnome-focus/REPORT.agentk.md` records real GNOME 50 launch and focus checks. Six Rust tests pass. The user previously confirms launcher rendering and a Ghostty launch.
- **Reverse if:** Agent proposal: replace this adapter if a supported public interface removes the extension requirement, or the final demo uses another platform.
- **Open:** The user has not separately specified reversal criteria or alternative preferences. Desktop-session Ghostty focus, global summon, tiling, and the final demo platform remain unverified. The user session needs sign-out/in before the new extension can enable.
- **Update (2026-11-01):** On continuation October 30, GNOME enabled the extension (`gnome-extension/README.agentk.md`, Installation). The user confirmed global summon with Ctrl+Alt+Space (`FOCUS_BAR.agentk.md`, Global summon). D-008 covers tiling. Desktop-session Ghostty focus is confirmed by the user; the final demo platform remains open.
