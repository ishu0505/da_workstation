---
id: D-014
status: accepted
date: 2026-11-02
supersedes: []
source_paths: [gnome-extension/extension.js, gnome-extension/layout.js, src/windows.rs, src/mcp.rs, src/sidebar.rs]
---

# D-014: Apps opened from DA Workstation tile themselves; bottom-bar placement

- **Decision:** Opening an app from DA Workstation (the typing box, an app row, or Muse's `open_app` and `new_window` tools) tiles it without a key press. Before a launch the app arms the extension's `StageNext(true)`; the next normal, non-dialog window that opens within 20 s joins the stage as the big window (its old place is added to the restore snapshot), or stages every window when nothing is tiled. Focusing an existing window from DA Workstation stages everything when nothing is tiled; when tiled, the focus change swaps it in. Apps opened outside DA Workstation are left alone. The app can also dock as a bottom bar (full width, 300 px tall): a header button switches between "Side panel" and "Bottom bar", saved in `$XDG_DATA_HOME/da-workstation/placement`; `PlaceSidebarAt(placement, activate)` docks it, and tiling then uses the space above the bar. Bottom mode shows three columns: sessions, the chat or results with the one typing box, and apps.
- **Why:** The user asked on November 2 for the bottom-bar mode ("can we add this") and said they should not have to press a tiling key: an app opened from DA Workstation should start in the tiled format.
- **Rejected:** Agent inference: tiling every new window on the desktop (it would move windows the user opened elsewhere, and dialogs).
- **Evidence:** Isolated GNOME run, 20 checks pass, including "an app opened through DA Workstation is tiled as the big window" (StageNext, launch a fifth app: it is big, four cards) and "bottom-bar mode docks along the bottom and tiles above it; left mode returns". Layout test covers the area above a bottom bar. Headless screenshots of bottom mode.
- **Reverse if:** Auto-tiling surprises the user; then keep it behind a setting.
- **Open:** Not yet seen on the user's desktop (needs sign-out/in).
