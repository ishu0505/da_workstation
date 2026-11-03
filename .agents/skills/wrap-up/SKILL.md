---
name: wrap-up
description: Finish a task so the docs stay true - run the checks, compare the branch's docs with its code, offer to refresh stale learning docs, offer catch-up for code written by hand, and write the explain-diff section for a pull request. Use at the end of every task, when the user says the work is done, and always before opening or updating a pull request.
---

# Wrap-up

The last step of every task. It keeps the human in the loop: it reports
what it found and asks before each larger change. Never commit unless the
user asks.

## 1. Checks

Run `sh .agents/kit/check.sh`. Fix what fails, and report what you fixed.
In a team repo (`MAPS="beside"`), a stale-wiring failure is fixed with
`sh .agents/kit/wire.sh`; never stage or commit a kit file there.
Problems marked `WARN` are not part of this task: list them, do not fix
them unless the user asks.

## 2. Docs of this branch

Find the base: the branch the work will merge into (`main` by default).
Run the `docs-sync` skill in check mode with scope `since <base>`. When it
finds drift caused by this task in kit docs, propose the fixes and apply the
ones the user approves (`docs-sync fix`). Show its "Your docs" table as it
is: those files belong to the people, so they fix them.

## 3. Learning docs

Read `$(git rev-parse --git-common-dir)/agent-kit/stale-learning` (the Stop
hook writes it) and `sh .agents/kit/stale.sh since <base>`. If learning docs
are stale, ask once: "N learning docs describe code this task changed.
Refresh them now?" On yes, run the `learning-docs` skill with scope `stale`,
then empty the stale-learning file. On no, leave the file; the next session
start shows it.

## 4. Code written by hand, and other people's commits

If `sh .agents/kit/queue.sh count` is above zero, tell the user and offer
the `catch-up` skill. Do not run it without a yes.

If `sh .agents/kit/pulled.sh count` is above zero, other people's commits
changed code since the docs were last checked. Tell the user and offer the
`docs-sync` skill with scope `pulled`. Do not run it without a yes.

## 5. Task file, and what this task learned

Run `sh .agents/kit/task.sh list`. If a task file (`.agents/work/`) belongs
to this work, read it, then:

1. For each entry under Decisions that should last (marked "(lineage)", or
   a choice about architecture, a library or a contract), offer the
   `lineage` skill. Do not run it without a yes.
2. Propose the map rows the task taught, from Found: a folder that was a
   trap ("Do not read unless asked"), a folder that looked right but was
   wrong ("Lookalikes"), or two files linked in a way search did not show
   ("Hidden links"). Show each row and the map it goes in; write only the
   rows the user approves. At most five rows per section in a map.
3. If Next or the Plan still has open steps, ask: close the task, or keep
   it paused? For paused, set `Status: paused` and make Next exact.
4. On close, run `sh .agents/kit/task.sh close <name>`.

With no task file, propose a map row only when this task hit a real trap,
at most two, each with the user's yes. Never add a row from a guess: each
one must come from something this task saw.

## 6. Pull request

When a pull request is being opened or updated, run the `explain-diff`
skill for `since <base>` and put its section in the pull request
description (replace the old section between the markers; never add a
second one). When the user asks for the plainer explanation, use
`explain-d-ez` instead; for an interactive page, `explain-d-in`.

## 7. Report

A short list: checks (pass or fixed), docs changed, learning docs
refreshed or still stale, catch-up items, other people's commits waiting
for docs-sync, the task file (closed, paused, or none) and map rows added,
and whether the pull request section was written.
