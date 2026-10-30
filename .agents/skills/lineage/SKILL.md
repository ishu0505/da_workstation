---
name: lineage
description: Record a decision or an experiment result in the repo's lineage (DECISIONS.agentk.md, one file per decision, and the experiment ledger), and bring the AGENTS.md maps up to date. Use when the user decides something about architecture, a model, a library, a stage or a tool; when an experiment or benchmark finishes; or when a folder, service or unit is added, moved, renamed or retired.
---

# Lineage

The lineage is the decisions and, when the repo has one, the experiment
ledger named by `LEDGER` in `.agents/agent-kit.conf`. Both are
append-mostly: history is kept, and a changed mind is a new entry that
supersedes the old one.

File names: `DOC_SUFFIX` in the same file (`.agentk.md`) ends every file
this skill writes; below, `DECISIONS<suffix>` means `DECISIONS.agentk.md`
(or `DECISIONS.md` when `DOC_SUFFIX` is empty, in repos set up before kit
0.6). Where decisions live depends on `DECISIONS_DIR`:

- set (e.g. `docs/decisions`): one file per decision,
  `<DECISIONS_DIR>/D-NNN-<short-slug><suffix>`, and one row for it in the
  index table of `DECISIONS<suffix>`;
- empty: every decision is an entry in `DECISIONS<suffix>` itself.

Write by `.agents/kit/writing-style.agentk.md`.

## 1. Name the change

State in one sentence what was decided or measured. Done when the sentence
names a single choice and what it replaces.

## 2. Grill

Ask the user, one question at a time, until each has an answer in their own
words:

- **Why** this choice, toward which of the repo's priorities (root
  `AGENTS.md`)?
- **What else** was considered, and why did it lose?
- **What evidence** supports it? A report path and its numbers, or "none yet".
- **What would reverse it?** A result, a cost, a scale.

Push back on an answer that restates the decision, cites no evidence, or
contradicts an earlier decision; quote that entry. Accept
"I don't know yet" and record it as open. Done when all four are answered or
explicitly open.

## 3. Write the decision

Take the next free id (the highest `D-NNN` in use, plus one) and today's
date. Record the user's reasoning as they gave it; mark any inference of
yours as yours.

- With `DECISIONS_DIR`: create the file from the kit's decision template
  (front matter `id`, `status`, `date`, `supersedes`, `source_paths`, then
  the sections). `source_paths` lists the code the decision governs, so
  `docs-sync` rereads it only when that code changes. Add the row
  `| [D-NNN](<path>) | <the choice> | <status> | <date> |` to `DECISIONS<suffix>`.
- Without it: append the entry to `DECISIONS<suffix>` using the template at its
  top.

When the decision replaces an earlier one, set the earlier one's status to
`superseded by D-NNN` (in its file and its index row) and change nothing
else in it.

**Moving to one file per decision.** When `DECISIONS_DIR` is set but
`DECISIONS<suffix>` still holds full entries, offer to split them: one file per
entry, unchanged text, and the index table. Do it only on the user's yes.

## 4. Update the experiment ledger

When an experiment was involved and `LEDGER` is set, add or update its row:
folder, question, scale (documents, questions, runs), outcome (a number, or
`pending: not run yet`), report path, decision id.

## 5. Update the maps

Check each map the change touches, from the root down to the unit: the
kit's `AGENTS.md`, or `AGENTS.agentk.md` in a team repo (`MAPS="beside"`,
where the team's `AGENTS.md` is never edited and `sh .agents/kit/wire.sh`
runs after a map changes):

- add, move or remove map rows so every listed path exists;
- add or reword an invariant the decision creates or ends;
- delete lines the change made stale.

Keep maps within `TOP_MAP_MAX` and `UNIT_MAP_MAX` lines (config). Detail
belongs in the unit's detail doc (`README.agentk.md`) or the report the map
points to. A new unit gets its map
from the `scaffold` skill.

## 6. Check and report

Run `sh .agents/kit/check.sh`. Tell the user which entries and lines
changed, and list every open question left from step 2.
