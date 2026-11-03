---
id: D-015
status: accepted
date: 2026-11-02
supersedes: [D-011]
source_paths: [gnome-extension/extension.js, src/sidebar.rs, src/switcher.rs, src/activity.rs, src/windows.rs]
---

# D-015: Chat-style left panel, apps as cards on the right, auto-tile every window

- **Decision:** The left panel holds conversations and commands only, after the Claude and ChatGPT desktop apps: a "＋ New task" button, a "Needs you" box only while a request waits, sessions grouped Today / Yesterday / Previous 7 days / Older (one line each: status icon, title, unread count or age; "Older" folds), and the one typing box with a footer (project folder, side/bottom switch, auto-tile switch). Typing `/` lists commands: /tile, /restore, /bottom, /side, /autotile on, /autotile off. Open apps are no longer listed or given chats in the panel; they live on the right as the stage's cards, each captioned with the app name and the window title (a browser's current page title), kept current as titles change. Apps stay reachable through the typing box. Auto-tiling is on by default: every new normal window on the current workspace, from any launcher including the GNOME menu, joins the stage as the big window (or stages everything); dialogs and other workspaces are left alone. `SetAutoStage` turns it off (then only apps opened from DA Workstation tile, D-014); the setting is kept in `$XDG_STATE_HOME/da-workstation/autostage`. When closing windows leaves fewer than two tiled, tiling ends with a full restore, so a leftover restore point never blocks the next tile. Supersedes D-011's apps list and app chats (its layout part was already superseded by D-012).
- **Why:** The user on November 2: any app opened, also from the GNOME menu, should be tiled; the right side should show app windows with labels and metadata such as the open web page; the left panel should keep only Muse sessions and commands, as it was "cluttered and difficult to use"; take inspiration from the Claude and ChatGPT desktop apps.
- **Rejected:** Agent inference: keeping the apps list in the left panel (duplicates the right-hand cards and was the main clutter).
- **Evidence:** Isolated GNOME run, 21 checks pass, including "with auto-tiling on, an app opened outside DA Workstation joins the stage". `switcher` tests cover `/` commands. Headless screenshots of the new panel, captions and the command list. The harnesses now use their own `XDG_STATE_HOME`; before this, a run had written "off" to the user's real auto-tile setting, which was removed (back to the default).
- **Reverse if:** Auto-tiling every window gets in the way (a quick calculator, a picture viewer); then default it off.
- **Open:** Not yet seen on the user's desktop (needs sign-out/in). Caption metadata is the window title only; no tab list.
