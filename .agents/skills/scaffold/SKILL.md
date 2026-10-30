---
name: scaffold
description: Create the agent context for a new unit (service, package, app, library, experiment) - a detail doc (README.agentk.md), an AGENTS.md map and pointer files - from the user's answers, and register it in the parent map. Use whenever the user starts a new unit, or a unit folder has no AGENTS.md (check.sh reports "has no AGENTS.md").
---

# Scaffold

A unit is a folder listed in `UNITS`, or a child of a folder listed in
`UNIT_PARENTS`, in `.agents/agent-kit.conf`.

**Map or row?** A unit gets its own map only when it is big enough:
`MAP_MIN_FILES` code files or `MAP_MIN_LINES` code lines, or it is listed in
`MAP_REQUIRED` (never when it is in `MAP_EXEMPT`). A smaller unit gets one
row in the nearest map above it (or its parent folder does), so agents can
find it without loading another file. Tell the user which case applies and
why; they may override it in the config. When a small unit grows past the
threshold, `check.sh` asks for its map.

A unit with its own map carries:

- `README<DOC_SUFFIX>` (`README.agentk.md`): the detail, for people and
  agents;
- `AGENTS.md`: a short map, at most `UNIT_MAP_MAX` lines, pointing into the
  detail doc;
- one pointer file per entry in `POINTER_FILES` (e.g. `CLAUDE.md`), each
  containing exactly `@AGENTS.md`.

If the new folder is not covered by `UNITS` or `UNIT_PARENTS`, ask the user
whether it is a unit, and add it to the config if so.

**In a team repo** (`MAPS="beside"`): the unit's map is `AGENTS.agentk.md`,
there are no pointer files, and a team `AGENTS.md` or `CLAUDE.md` in the
folder is read first and never edited. Write only what the team's file does
not say. Then run `sh .agents/kit/wire.sh`, which connects the new map to
each agent with local files.

**The unit's own `README.md` stays theirs.** If the folder already has a
`README.md` (or any Markdown file without the suffix), read it: it answers
many of the questions below. Link to it from the map and the detail doc.
Never copy it, edit it or move it. With `DOC_SUFFIX` empty (a repo set up
before kit 0.6), the detail doc is `README.md`; then write it only when the
folder has none.

## 1. Ask

Ask as `.agents/kit/writing-style.agentk.md` says: one question at a time,
with your proposed answer first and "Something else: type your own answer"
last. Accept "not decided yet" and write it down as open.

Service, package or app:
- what it does, and what it must never do;
- its entry point (HTTP route, queue, CLI, library API) and how it is run
  (container, port, command);
- the rules a change must not break;
- how to build, run and test it, with the real commands.

Experiment or benchmark:
- the question it answers, in one sentence;
- inputs, and where the expected answers come from (hand-written, never
  model output unless the user says so, and then labelled as such);
- the metric and what counts as better;
- the decision it feeds, or "none yet";
- the run command, and what is not covered.

## 2. Write

Create the folder and the files. Write the detail doc by
`.agents/kit/writing-style.agentk.md`. Start the map's rows from
`sh .agents/kit/map-rows.sh <folder>`, which lists the subfolders and entry
files from the tree; you write only the "What" column. `AGENTS.md` follows
this shape (from
`templates/docs/AGENTS.unit.md` in agent_dev_kit):

```
# <name>

<one line: what it does / the question it answers>
Details: `README.agentk.md`. Shared rules: the maps above this folder.

## Map
| Path | What |
|---|---|
| `<path>` | <one line> |

## Run
<command>

## Rules
- <rules for this folder only; never repeat the parent's>

## Further reading
- `README.agentk.md` — <what is in it>
- `README.md` — <the people's own README; only when it exists>
```

Only list paths that exist; the checks fail on a missing one.

## 3. Register

- Add a row for the unit to the parent folder's `AGENTS.md` map (or the
  root map). Point to it; do not copy its content.
- If the unit sits beside the experiment ledger (`LEDGER`), add its row
  with outcome `pending: not run yet` through the `lineage` skill.
- If the unit is new architecture, record the decision with `lineage`.

## 4. Check

Run `sh .agents/kit/check.sh`. Report the files created and any answers
left open.
