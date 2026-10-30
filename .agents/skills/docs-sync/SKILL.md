---
name: docs-sync
description: Check whether the code and the docs agents and humans read (AGENTS.md maps, READMEs, decisions, learning docs) still agree, and optionally make the docs match the code. Use when the user asks "are the docs in sync", before merging a branch (wrap-up runs it), after a refactor or a pull of other people's changes, or when an agent seems misled by a doc. Modes - check (report only) or fix. Scope - since <git ref> (default the base branch), a unit path, or all.
---

# Docs sync

`.agents/kit/check.sh` catches mechanical drift: a long map, a dead path, a
missing file. It cannot see that a README names an old default or that a
learning doc describes a removed step. This skill does that reading.

**Arguments.** Mode: `check` (default; change nothing) or `fix`. Scope:
`since <ref>` (default `since main`, or the repo's default branch: only
what this branch changed), a unit path, or `all` (every doc against its own
last update; slower). Settings come from `.agents/agent-kit.conf`.

## 1. Run the checks

Run `sh .agents/kit/check.sh`. Every failure goes into the report.

## 2. Find the candidates (git only, no reading yet)

Run `sh .agents/kit/stale.sh since <ref>` (or `sh .agents/kit/stale.sh` for
scope `all`). It lists each doc whose **anchors** changed, with the files:

- a map: the paths it names (a named folder counts only when a file in it
  is added, removed or renamed);
- a detail doc (`README.agentk.md`) or a README people wrote
  (`README.md`): the paths in its `<!-- agent-kit: source_paths ... -->`
  line, else its folder;
- a learning doc: its `source_paths` since its `source_commit`;
- an accepted decision: its `source_paths` (with `DECISIONS_DIR`).

Only these docs are read in step 3. For a unit path, keep the docs inside
it. With `DECISIONS_DIR` empty, also read each `accepted` entry of
the decision index that names a changed file. When a kit detail doc has no
anchor line and keeps showing up for unrelated changes, add one in `fix`
mode.

The third column says who owns each doc: `kit` (the kit wrote it: maps,
pointer files, files ending in `.agentk.md`) or `yours` (people wrote it).
Read both. Fix only `kit` docs; see `.agents/kit/writing-style.agentk.md`,
"Which files you may edit".

## 3. Read and compare

For each candidate, read the doc and the code diff since the doc's commit.
List every claim that is now false:

- paths, commands and scripts (does the file or target exist?);
- env vars and settings (grep the code for each name the doc gives);
- defaults, limits, ports, versions;
- interfaces: function names, routes, message fields, CLI flags;
- flow steps in learning docs (does each named function still do that?);
- invariants: search for a violation (e.g. a provider SDK imported outside
  the one module allowed to call it).

A claim you cannot confirm or refute is "unverified", not "wrong".

**In a team repo** (`MAPS="beside"`), also compare each of your maps
(`AGENTS.agentk.md`) with the team's `AGENTS.md` / `CLAUDE.md` in the same
folder and above it. List every place where yours contradicts theirs or
repeats them. The team wins: the fix is always in your map.

## 4. Report

Two tables, each grouped by unit:

**Kit docs** (`fix` mode changes these):

| Doc | Claim | What the code does | Evidence | Proposed fix |
|---|---|---|---|---|

**Conflicts with the team's rules** (team repo only; fix your map):

| Your map | Your line | Team file | Their line | Fix in your map |
|---|---|---|---|---|

**Your docs: report only** (the people fix these themselves):

| Doc | Line | Says | What the code does | Evidence | Sentence to paste |
|---|---|---|---|---|---|

Evidence is `file:line`. End with: docs checked, docs in sync, docs with
drift, unverified claims. In `check` mode, stop here and change nothing.

## 5. Fix (`fix` mode only)

- **Docs follow the code.** Never change code to match a doc.
- A change that breaks an invariant or an `accepted` decision is not fixed
  silently. Ask the user: was the code change intended? If yes, record a new
  decision with the `lineage` skill and update the invariant. If no, it is a
  bug: leave the doc, and report the bug.
- Rewrite stale map lines and kit detail docs (`README.agentk.md`) by
  `.agents/kit/writing-style.agentk.md`. Keep maps within their line limits.
- **Never edit a `yours` doc** (`README.md`, `CONTRIBUTING.md`, `docs/*.md`
  and other Markdown the kit did not write). Leave its rows in the "Your
  docs" table for the people to fix.
- Refresh stale learning docs with the `learning-docs` skill (it updates
  each `source_commit`).
- Run `sh .agents/kit/check.sh` again and show that it passes. Report every
  file changed.
