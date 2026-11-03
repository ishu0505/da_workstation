---
id: D-008
status: accepted
date: 2026-10-30
supersedes: []
source_paths: [src/main.rs, src/windows.rs, gnome-extension/extension.js, experiments/gnome-focus/verify.py]
---

# D-008: Add first native GNOME tiling controls

- **Decision:** Extend the existing GNOME adapter with master/stack tiling and restoration. Ctrl+T requests tiling; Ctrl+R requests restoration. This supplements D-007.
- **Why:** The user approves native tiling as the next step after global summon. Native tiling is part of the required working flow.
- **Rejected:** Agent inference: a separate window-management service adds complexity before this adapter is proven. The user's alternative preferences remain open.
- **Evidence:** `experiments/gnome-focus/REPORT.agentk.md` records two real GTK windows within the work area without overlap, followed by exact rectangle restoration. Six Rust tests pass. The updated extension is installed, but GNOME rejects live reload as unimplemented.
- **Reverse if:** Agent proposal: change the adapter if desktop acceptance fails or the final demo platform changes. User-specific reversal criteria remain open.
- **Open:** Desktop Ctrl+T/Ctrl+R, three-window geometry, minimum-size constraints, and maximized-window recovery need verification. The floating bar does not reserve screen space. Restoration is in memory and does not survive extension unload. The final demo platform remains open.
- **Update (2026-11-01):** After sign-out/in, the user confirms in the real desktop that Ctrl+T tiles the windows and Ctrl+R restores them. Extension errors now show their own reason in the bar. Three-window geometry, minimum sizes and maximized-window recovery are still not separately checked. Agent proposal, from a review of existing GNOME tiling extensions: keep this adapter and borrow ideas, not code (see `FOCUS_BAR.agentk.md`, Tiling ideas).
