---
name: learning-docs
description: Write or refresh the learning docs (LEARNING_DIR, default learning_docs/) that teach a person the whole architecture and each unit and flow in ASD-STE100 Simplified Technical English, written for an intern, with real code references, invented examples, Mermaid diagrams and easy/medium/hard quizzes with hidden answers. Whether they are committed follows SHARE in .agents/agent-kit.conf. Use when the user asks for learning docs, an explanation of the system for a newcomer, when a unit is added, or when wrap-up or docs-sync reports a stale learning doc. Scope - all, a unit, a flow, or stale.
---

# Learning docs

The folder is `LEARNING_DIR` in `.agents/agent-kit.conf` (default
`learning_docs/`). It explains the system to a person who has never seen
it. Whether it is pushed depends on `SHARE` in the same file:

- `agent` or `none`: the docs stay local. **Never commit or push them.**
  Before writing, confirm `git check-ignore -q <LEARNING_DIR>/probe.md`
  succeeds; if not, run the kit's `install.sh` first. Committed docs and
  maps never link into the folder, since a clone does not have it.
- `all` or `learning`: the docs are committed like code. Stage them with
  the change they describe. Never put secrets, customer data or private
  hosts in them, even in examples.

**Scope argument:** `all` (default for a first run), a unit path, a flow
name, or `stale` (only docs whose code changed; see "Staying current").

## 1. Plan the files

Read the root `AGENTS.md`, every unit map, and the code entry points. Then
propose the file list and wait for the user's OK. Every file name ends in
`DOC_SUFFIX` from the config (`.agentk.md`; plain `.md` when it is empty):

- `00-start-here.agentk.md` — reading order, one line per file.
- `00-overview.agentk.md` — the whole system: what it does, the units, how they
  talk, one end-to-end diagram, where to start reading the code.
- One file per unit (service, package, component), in the order data flows
  through them: `01-api.agentk.md`, `02-worker.agentk.md`, ...
- One file per important flow that crosses units: API request, queue or
  worker job, event stream, data pipeline, CLI command, retry and failure
  paths: `05-upload-to-result-flow.agentk.md`, ...

## 2. Read before you write

For each file, read every function you will describe, end to end. Write only
what the code shows. When the code does not show something (who calls this
service, production settings), write "not shown in this repo". Mark an
illustrative sequence as illustrative.

## 3. Write each file

Start with front matter recording its sources:

```
---
title: One worker job from start to finish
units: [worker, queue]
source_paths: [src/worker/, src/queue/client.go]
source_commit: <git rev-parse --short HEAD>
---
```

Sections, in this order (leave out one that does not apply):

1. **Simple summary** — three to five sentences: what it does, its input,
   its output, and what it does not do.
2. **A concrete example** — a realistic request, message, command or input.
   Say it is invented. Use fake ids, hosts and accounts; never real secrets,
   customer data or production addresses.
3. **The exact path** — numbered steps, one action each, in execution order.
   Name the real function and link its file and line, e.g.
   ``[`process_message()`](../src/worker/main.py#L120)``. Say what each step
   checks, changes or sends, and where it can fail.
4. **Diagram** — a Mermaid `flowchart` or `sequenceDiagram` of the same
   steps, error paths included.
5. **What comes out** — the shape of the reply, stored record or event, with
   an example.
6. **Failure and edge cases** — retries, timeouts, partial success,
   duplicates, what is and is not atomic. Correct a likely wrong assumption
   in plain words ("A log line here does not prove the reply arrived").
7. **Next** — links to the files to read next.
8. **Quiz** — at least six questions: two **easy** (recall a fact), two
   **medium** (trace a flow or explain a reason), two **hard** (predict
   behaviour in a failure or edge case, or compare two designs). Test
   understanding of the project, not trivia. Hide the answers:

   ```
   <details>
   <summary>Check your answers</summary>

   1. **Easy.** ... (see `src/worker/main.py`, `process_message()`)
   </details>
   ```

   Each answer gives the reason and the code reference.

The overview uses the same sections at system level: the example follows one
request through every unit.

## 4. Language

Follow `.agents/kit/writing-style.agentk.md`: ASD-STE100 Simplified Technical
English, written for an intern on day one. Code names, paths and API fields
stay exactly as in the code.

## 5. Staying current

A learning doc is stale when
`git diff --stat <source_commit> -- <source_paths>` lists changes. The Stop
hook also lists the docs a session made stale in
`$(git rev-parse --git-common-dir)/agent-kit/stale-learning`; `wrap-up` asks
the user to refresh them at the end of the task. With scope `stale`, take
both lists, rewrite only the affected sections, update `source_commit`,
remove the refreshed docs from the stale-learning file, and report what
changed.

## 6. Check and report

- Every relative code link resolves to an existing file.
- Every file has the quiz with hidden answers.
- With `SHARE` `agent` or `none`: `git status --short` shows nothing under
  `LEARNING_DIR`.

Report the files written and any claim marked "not shown in this repo".
