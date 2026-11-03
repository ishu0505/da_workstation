---
id: D-006
status: accepted
date: 2026-10-30
supersedes: D-001
source_paths: [Cargo.toml, src/main.rs, src/apps.rs]
---

# D-006: Start the Rust app launcher October 30

- **Decision:** Start a small Rust focus bar today that opens installed apps and terminals. Retain native tiling, the focus bar, and real Muse delegation as required outcomes. Finish November 2; submit November 3 only after explicit authorization. All dates use Asia/Singapore.
- **Why:** The user wants a working first result today. Agent inference: a bounded launcher advances the required command flow without waiting for window-manager integration.
- **Rejected:** Continuing planning only does not meet the new request. Agent inference: implementing tiling, routing, and Muse together would enlarge this first slice. Other alternatives remain open.
- **Evidence:** The October 30 user request asks for a Rust focus bar that opens apps and terminals. Local inspection finds Fedora GNOME Wayland, Rust 1.99.1, desktop libraries, and Ghostty. Runtime verification is recorded in FOCUS_BAR.agentk.md.
- **Reverse if:** An explicit user change replaces the schedule or scope. Report any blocker to the required flow before changing frameworks or dropping requirements.
- **Open:** Final demo platform, global summon shortcut, window-control integration, and Muse protocol remain unverified. GPUI remains the chosen candidate for this first build, subject to runtime evidence.
