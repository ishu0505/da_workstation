#!/bin/sh
# SessionStart hook: records where the session starts, so the Stop hook sees
# only this session's changes, and tells the agent in a line or two what is
# waiting: commits that changed code without its docs, commits other people
# pushed that changed code since the docs were last checked, stale learning
# docs, and open task files (.agents/work/).
# Prints nothing when nothing waits.
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$ROOT" || exit 0
input=$(cat)
printf '%s' "$input" | sh .agents/kit/stop-hook.sh --baseline --repo "$ROOT" >/dev/null 2>&1
# Beside mode (a team repo): refresh the local wiring of your maps first.
# In a workspace, wire.sh also points Codex to the workspace map.
sh .agents/kit/wire.sh --quiet >/dev/null 2>&1
pending=$(sh .agents/kit/queue.sh count 2>/dev/null)
[ "${pending:-0}" -gt 0 ] 2>/dev/null &&
  echo "agent-kit: $pending commit(s) changed code without its docs. Tell the user once, and run the catch-up skill if they agree."
pulled=$(sh .agents/kit/pulled.sh status 2>/dev/null | head -1)
[ -n "$pulled" ] &&
  echo "agent-kit: $pulled Tell the user once, and offer the docs-sync skill with scope pulled (it checks the docs and learning docs against those commits)."
stale="$(git rev-parse --git-common-dir)/agent-kit/stale-learning"
[ -s "$stale" ] &&
  echo "agent-kit: $(wc -l < "$stale" | tr -d ' ') learning doc(s) are stale; the wrap-up skill refreshes them at the end of the task."
tasks=$(sh .agents/kit/task.sh list 2>/dev/null | awk -F '\t' '
  NR <= 3 { printf "%s%s (%s, branch %s%s)", sep, $1, $2, $3, ($4 == "" ? "" : ", next: " $4); sep = "; " }
  END { if (NR > 3) printf "; and %d more", NR - 3 }')
[ -n "$tasks" ] &&
  echo "agent-kit: open task file(s) in .agents/work/: $tasks. Ask the user once whether to continue one; if yes, read its file first and work from it."
exit 0
