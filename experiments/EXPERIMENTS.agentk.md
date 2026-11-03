# Experiment ledger

One row per experiment folder beside this file, updated by the `lineage`
skill when a run finishes. Every row has an outcome; one not run yet says
`pending: not run yet`. "Decision" is an id in `DECISIONS.agentk.md`.

| Folder | Question | Scale | Outcome | Report | Decision |
|---|---|---|---|---|---|
| `gnome-focus` | Can Rust focus existing GNOME windows without duplicates? | GNOME 50; 4 native GTK apps; user-desktop Ghostty | isolated launch/focus, 2- and 4-window tiling, recent-N, directional focus, restore after reload, sidebar docking and glow frames pass (15 checks, November 2); user confirms Ghostty focus and Ctrl+T/Ctrl+R | [Report](gnome-focus/REPORT.agentk.md) | D-007, D-008, D-009 |
| (no folder; scratch run) | Does the installed Muse run headless, and what does it emit? | Muse Code 1.4.14; one `meta` run, two `echo` runs | `exec --json` emits JSONL (`run.output.delta`, `run.terminal.completed/failed`); the `meta` run fails with 402 billing on an API key and works with a subscription login; a reused `--session-id` keeps context; `muse schema` has approval methods on `muse serve` (November 2) | — | D-009 |
| (no folder; scratch probe) | Can the user answer a real Muse approval? | Muse Code 1.4.14 `muse serve`; shell approval, reject and allow; resume; stop | judge off: request arrives; reject leaves no file (`abort`, by user); allow creates it; resume keeps context; stop cancels in 13 ms (November 2) | — | D-010 |
| (no folder; scratch probes) | How does the Muse flow fail and recover? | Muse Code 1.4.14 `muse serve`; no credentials; client closes mid-command; 4 real adapter tests | no credentials: `turn/completed` with `error.kind` `authRequired` ("not logged in: run /login"), shown with a sign-in step; client gone: `muse serve` exits in 0.1 s (`stdin_closed`) and its running command does not finish; stop, reject/allow, resume pass again (November 2) | — | D-010 |
