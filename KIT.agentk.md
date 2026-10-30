# Using the agent kit in this repo

This repo uses [agent_dev_kit](https://github.com/ishu0505/agent_dev_kit). It gives coding agents short
maps (`AGENTS.md`), keeps the reason behind each choice
(`DECISIONS.agentk.md`), and checks that the docs stay true.

Every Markdown file the kit writes ends in `.agentk.md`, except `AGENTS.md`,
`CLAUDE.md` and `GEMINI.md`, which agents need by those exact names. The kit
never edits your other Markdown files, such as `README.md`: it reads them,
and tells you when one no longer matches the code.

| Situation | Do this |
|---|---|
| Fresh clone, once | `git config core.hooksPath .githooks` |
| Quick check (seconds, no AI) | `sh .agents/kit/check.sh` |
| End of a task, or before a PR | ask the agent: `wrap-up` |
| You wrote code by hand | `catch-up`: the agent explains it, proposes docs and docstrings, you approve |
| Explain a branch or PR, with a quiz | `explain-diff` (`explain-diff --post` adds it to the PR) |
| Are code and docs in sync? | `docs-sync check` |
| Make the docs match the code | `docs-sync fix` |
| Learn the system | `learning-docs` (writes `learning_docs/`; pushed only in share mode `all` or `learning`) |
| Tidy agent and doc files | `agent-kit cleanup` |
| Record a decision or result | `lineage` |
| Which docs may be stale? | `sh .agents/kit/stale.sh` (read only) |
| A commit that needs no docs | put `[no-docs]` in its message |
| Add a service, package or experiment | `scaffold` |
| Edited a skill (copy mode) | `sh .agents/kit/sync-skills.sh` |
| Newer kit version | in the kit folder `git pull`, then ask the agent: `agent-kit upgrade` |

On Windows, run the shell commands in Git Bash. How to call a skill: Claude Code and agy `/docs-sync check`; Codex `$docs-sync check`. Cursor,
Antigravity and other agents that load `.agents/skills`: name the skill.
Any agent: "Read `.agents/skills/docs-sync/SKILL.md` and run it in check
mode."
