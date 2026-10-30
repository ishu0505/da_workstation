#!/bin/sh
# SessionStart hook: records where the session starts, so the Stop hook sees
# only this session's changes, and tells the agent in a line or two what is
# waiting: commits that changed code without its docs, stale learning docs.
# Prints nothing when nothing waits.
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$ROOT" || exit 0
input=$(cat)
printf '%s' "$input" | sh .agents/kit/stop-hook.sh --baseline >/dev/null 2>&1
# Beside mode (a team repo): refresh the local wiring of your maps first.
grep -qs '^MAPS="beside"' .agents/agent-kit.conf && sh .agents/kit/wire.sh --quiet >/dev/null 2>&1
pending=$(sh .agents/kit/queue.sh count 2>/dev/null)
[ "${pending:-0}" -gt 0 ] 2>/dev/null &&
  echo "agent-kit: $pending commit(s) changed code without its docs. Tell the user once, and run the catch-up skill if they agree."
stale="$(git rev-parse --git-common-dir)/agent-kit/stale-learning"
[ -s "$stale" ] &&
  echo "agent-kit: $(wc -l < "$stale" | tr -d ' ') learning doc(s) are stale; the wrap-up skill refreshes them at the end of the task."
exit 0
