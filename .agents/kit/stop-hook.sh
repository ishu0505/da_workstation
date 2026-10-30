#!/bin/sh
# Stop hook entry point for every agent. Finds the repo from wherever the
# agent runs, finds Python 3 under any of its usual names, and lets the stop
# through when there is none. Arguments go to docs_reminder.py.
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
for python in python3 python py; do
  if command -v "$python" >/dev/null 2>&1 &&
     "$python" -c 'import sys; sys.exit(sys.version_info < (3, 9))' 2>/dev/null; then
    exec "$python" "$ROOT/.agents/kit/docs_reminder.py" "$@"
  fi
done
exit 0
