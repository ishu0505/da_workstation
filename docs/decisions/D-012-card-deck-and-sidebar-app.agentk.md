---
id: D-012
status: superseded in part by D-013
date: 2026-11-02
supersedes: [D-011]
source_paths: [gnome-extension/layout.js, gnome-extension/extension.js, src/sidebar.rs, src/switcher.rs, src/main.rs, scripts/install-shortcut.py]
---

# D-012: Card deck tiling; the sidebar is the whole app

- **Decision:** Ctrl+T arranges tiled windows as a card deck: every card has the same near-full size of the area beside the sidebar (at least 900 px wide when the screen allows), offset left to right by 64 px (at least 40 px) so a strip of each window stays visible. Clicking a card, or any focus change, brings it to the front; nearer cards sit above farther ones. Nothing moves or resizes on focus, and hovering does nothing. The sidebar becomes the whole app: a search box ("jump to") finds sessions, open apps and installed apps (Enter opens, focuses or launches); Ctrl+T, Ctrl+2–9 and Ctrl+R work in the sidebar; Ctrl+Alt+Space runs `--sidebar --search`. The home view adds a "Needs you" inbox with Allow once and Reject, sorts sessions by urgency, folds finished sessions beyond five, and shows chat messages Slack-style (avatar, name, time). The floating bar stays in the code but no shortcut opens it. Supersedes D-011's focus columns and hover; D-011's Slack-style lists and app chats stand.
- **Why:** On November 2 the user saw focus columns fail on the real desktop (Chrome refuses narrow widths, so windows piled up on the right) and asked that the focused window expand only on click and that every window keep desktop proportions "as if they were full screen". The user also asked to drop the separate launcher and make the sidebar the entire app, and to make the panel more intuitive, with online inspiration. Patterns adopted from agent dashboards (attnbox, Opsnook) and Slack: needs-attention first, inline approve, collapse finished items.
- **Rejected:** Agent inference: focus columns with learned minimum widths (D-011). They depend on each app's minimum width and still overlap, and hover focus moved windows under the pointer.
- **Evidence:** `gjs -m experiments/gnome-focus/layout_test.js` passes (same-size cards inside the area, visible strips, stacking order). The isolated GNOME run passes 18 checks, including "focusing a card brings it to the front without moving or resizing any card". Headless screenshots of the deck, search results and Slack-style chat, November 2. 31 Rust tests pass.
- **Reverse if:** The user finds overlapping cards harder to use than side-by-side windows; then offer side-by-side for two windows only.
- **Open:** Clicking a strip was not verified in the headless shell (its virtual pointer left no window focused); check on the user's desktop. Handoff section 5's "do not overlap" gate does not hold for a deck by design.
