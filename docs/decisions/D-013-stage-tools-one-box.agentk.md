---
id: D-013
status: accepted
date: 2026-11-02
supersedes: [D-012]
source_paths: [gnome-extension/layout.js, gnome-extension/extension.js, src/mcp.rs, src/provider/muse_serve.rs, src/sidebar.rs, src/switcher.rs, scripts/install-shortcut.py]
---

# D-013: Stage tiling, desktop tools for Muse, one typing box

- **Decision:**
  1. Tiling is a stage: one big window beside the sidebar, the others as small cards in a right-hand column (at most 240 px, a fifth of the width on small screens). Every tiled window keeps the big size; a card is the same window drawn scaled (Clutter scale and translation), so apps keep desktop proportions and are never resized on a switch. A click target over each card catches the click (the app never gets it) and focuses that window, which then swaps with the big one (220 ms animation). Ctrl+Super+arrows cycle. Cards hide in the Activities overview and on other workspaces. Desktop-wide keys: Ctrl+Super+T tiles, Ctrl+Super+R restores (plain Ctrl+T belongs to apps).
  2. Muse gets the workstation's tools: `da-workstation --mcp` is a stdio MCP server (list_windows, find_apps, open_app, new_window, focus_window, tile, restore_layout, focus_direction, open_url), passed on every `session/start` and `session/resume` as `config.mcpServers` with the `sessionMcp` capability and the desktop environment variables. The adapter answers approval requests for these tools itself (allow), so the user is not asked; shell commands and file edits still ask. The client also declares `userInputDialogs: false` (the sidebar has no question dialog yet).
  3. One typing box: on the home view it searches sessions, open apps and installed apps and offers "Ask Muse"; a short query (three words or fewer, no `@folder`) puts matches first, a sentence puts Ask Muse first. In a chat it sends follow-ups. The separate search box is gone; Ctrl+K and Ctrl+Alt+Space focus the box on the home view.
  Supersedes D-012's card deck and search box; D-012's sidebar-as-app, Needs-you inbox, urgency order and Slack-style chat stand.
- **Why:** The user on November 2 (15:20): the deck "is just overlapping windows; there should be one big window and other small windows like cards"; Ctrl+T opened a new tab in apps; Muse "should have access to all the tools of da_workstation" in natural language, as programmatic tools, without permission prompts; and "I prefer having only one typing space".
- **Rejected:** Agent inference: resizing card windows small (apps switch to narrow layouts or refuse); letting clicks reach scaled windows (the click would hit the app's own buttons). Adding allow rules to the user's Muse settings: the approval rules format is undocumented, and the client already sees each request.
- **Evidence:** `layout_test.js` passes (big window at least 1,200 px beside the sidebar on 1920 px; cards keep proportions, no overlap). Isolated GNOME run: 18 checks pass, including "clicking or focusing a card makes it the big window; no window is resized". Real Muse test `real_muse_tools_run_without_asking`: Muse called `list_windows` with no user prompt and reported the user's 4 windows. 33 Rust tests pass. Headless screenshots of the stage and the single box.
- **Reverse if:** Muse documents tool approval rules (then configure them instead of auto-answering), or the user wants the bottom-bar mode (planned as a placement option, not built).
- **Open:** A real click on a card was not verified headless (virtual pointer); check on the desktop. Browser use is "open a URL" only; no page reading or clicking.
