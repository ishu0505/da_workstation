---
id: D-011
status: superseded
date: 2026-11-02
supersedes: []
source_paths: [gnome-extension/layout.js, gnome-extension/extension.js, src/sidebar.rs, src/activity.rs, src/provider/muse_serve.rs]
---

# D-011: Focus columns with hover-to-expand; Slack-style sidebar

- **Decision:** Ctrl+T arranges tiled windows as focus columns instead of master/stack: one column per window, the focused window wide (up to 1,200 px, at least 640 px), the others slim. A focus change, or the pointer resting 300 ms on a tiled window, focuses and expands that window. The extension learns each window's minimum width when the window refuses a smaller size, and arranges again. When the minimums cannot fit, slim windows tuck partly under the expanded one, inside the work area. The sidebar becomes a Slack-style list: agent sessions (one per task) and apps (grouped per app), each opening its own chat; an app's chat shows its windows with Focus buttons and an activity timeline from window changes. Changes D-008's master/stack layout; D-008's restore, sidebar space and directional focus stand.
- **Why:** The user asked on November 2: hovering a window should expand it to most of the screen, the focused app should keep desktop dimensions ("not narrow phone"), and the left panel should work like Slack, with separate sections for agents and apps, each like a chat.
- **Rejected:** Agent inference: swapping the hovered window into the master slot. The window leaves the pointer and the one that arrives under it expands next, so the layout flips. Focus columns keep each expanded column over its old place (tested for every transition), and hover waits for the pointer to move after a layout change.
- **Evidence:** `gjs -m experiments/gnome-focus/layout_test.js` passes (every expand transition covers its old place; minimums; tucking). The isolated GNOME run passes 18 checks, including expand-on-focus and tucking beside the docked sidebar. A virtual-pointer run expanded the hovered window and the layout then stayed still. Sidebar screenshots in a headless shell, November 2.
- **Reverse if:** Hover focus steals focus during normal work; then keep expand-on-focus and drop the hover timer.
- **Open:** Handoff section 5's "do not overlap" gate now holds only when the windows' minimums fit; tucked windows overlap under the focused one. Not yet seen on the user's desktop (needs sign-out/in). App chats keep activity only while the sidebar runs.
