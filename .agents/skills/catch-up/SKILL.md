---
name: catch-up
description: Explain code that was written by hand (or committed without its docs), then propose the doc, map, docstring and comment updates for the user to approve. Use when the session start says commits changed code without its docs, when the user says they wrote code themselves, or when the user asks to document recent work. Scope - the queue (default), uncommitted changes, or since <ref>.
---

# Catch-up

People write code by hand too. This skill helps the user understand that
code, and brings the docs up to date with it. **The user approves every
change. Never change code behaviour, never commit.**

Write every explanation and doc by `.agents/kit/writing-style.agentk.md`.

## 1. Find the changes

- Default: the queue. `sh .agents/kit/queue.sh list` prints one line per
  commit: id, date, number of files without docs, subject.
- `uncommitted`: `git diff HEAD` and `git status --short`.
- `since <ref>`: `git log --oneline <ref>..HEAD`.

Tell the user what you found in one short list, and ask which to start
with. Group commits that change the same unit.

## 2. Explain each group

Read the diff (`git show <commit>`) and the code around it. Then explain:

- **What changed**, in two to four plain sentences.
- **Why it matters**: what a user of the code now sees differently.
- **What to watch**: a risk, an edge case, a missing test. Say it only when
  the code shows it.

When the code does not show the intent, ask the user one question at a
time. Offer: "Give me one line about what this is for, and I will write
the comment or docstring from it."

## 3. Propose one patch per group

Show the proposal as a diff, in these parts:

1. **Maps and kit detail docs** (`AGENTS.md`, or `AGENTS.agentk.md` in a
   team repo; `README.agentk.md`). Rows
   for new or moved paths, changed commands, defaults, env vars and
   interfaces. Keep maps within their line limits.
2. **Docstrings and comments** in the changed code, following the comment
   rules in the writing style. Improve an existing docstring rather than
   replacing it. Add a comment only where the reason is not obvious.
3. **Decisions.** When the change makes or ends a choice, say so and offer
   the `lineage` skill.
4. **Learning docs.** List the learning docs whose `source_paths` cover the
   change; `wrap-up` refreshes them.
5. **Notes on your docs** (never part of the patch). When a `README.md` or
   other Markdown file the kit did not write now says something false, name
   the file and line and give a sentence the user can paste in.

Ask with choices, as the writing style says: apply all (recommended), apply
some (name them), change something, skip, or "something else: type it".

## 4. Apply only what the user approved

- Edit only comments and docstrings in code files. Check that with
  `git diff -- <file>`: every changed line in a code file is a comment or a
  docstring. If the repo has a check command (root `AGENTS.md`), run it.
- Run `sh .agents/kit/check.sh`.
- Remove the handled commits from the queue:
  `sh .agents/kit/queue.sh done <commit>...`. A skipped commit is removed
  too, when the user says so.

## 5. Report

The files changed, the commits handled, and anything left open. Remind the
user to review and commit; do not commit for them.
