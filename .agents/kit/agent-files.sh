#!/bin/sh
# Who owns the agent files already in this repo? Read only; it changes
# nothing. The setup runs it first, to ask "personal or team repo?" and what
# to do with each existing AGENTS.md / CLAUDE.md.
#
# Usage: sh agent-files.sh [repo]   (before the kit is installed, from
#        agent_dev_kit/templates/kit/; after, from .agents/kit/)
#
# It prints a guess for the repo (team or personal: authors, remote, your
# share of the commits), then one block per agent file: size, headings, git
# state, whether it is on the remote, who wrote it, and a guess:
#   YOURS          not in git, or committed only on this computer by you
#   PROBABLY YOURS on the remote, but only you wrote it
#   TEAM FILE      on the remote, and other people wrote it
#   KIT            made by agent-kit (wire.sh) on this computer
KIT_LIB="$(cd "$(dirname "$0")" && pwd)/lib.sh"  # before cd: $0 may be relative
ROOT=$(git -C "${1:-.}" rev-parse --show-toplevel 2>/dev/null) || {
  echo "agent-files: not inside a git repository"; exit 1; }
cd "$ROOT" || exit 1
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
. "$KIT_LIB" && load_conf
ME=$(git config user.email)

# --- The repo ------------------------------------------------------------------
total=$(git rev-list --count HEAD 2>/dev/null || echo 0)
authors=$(git log --format=%ae 2>/dev/null | sort -u | grep -c .)
mine=0; [ -n "$ME" ] && mine=$(git log --format=%ae 2>/dev/null | grep -cxF "$ME")
remote=$(git remote get-url "$(git remote | head -1)" 2>/dev/null)
echo "Repo"
echo "  commits:  $total, by $authors author(s); yours: $mine (${ME:-no user.email set})"
echo "  remote:   ${remote:-none}"
if [ "$authors" -gt 1 ]; then
  echo "  guess:    TEAM REPO: $authors people committed here"
elif [ "$total" -gt 0 ] && [ "$mine" -lt "$total" ]; then
  echo "  guess:    TEAM REPO: someone else made the commits here"
elif [ "$total" -eq 0 ]; then
  echo "  guess:    PERSONAL REPO: no commits yet"
else
  echo "  guess:    PERSONAL REPO: only one author"
fi
echo

# --- Each agent file ---------------------------------------------------------------
set -- -name .git
for skip in $SKIP_DIRS; do set -- "$@" -o -name "$skip"; done
files=$(find . \( "$@" \) -prune -o -type f \( -name AGENTS.md -o -name CLAUDE.md -o -name GEMINI.md \
  -o -name CLAUDE.local.md -o -name AGENTS.override.md -o -name .cursorrules -o -path './.claude/CLAUDE.md' \
  -o -path './.cursor/rules/*' -o -path './.github/copilot-instructions.md' -o -path './.gemini/settings.json' \) -print |
  sed 's|^\./||' | LC_ALL=C sort)
if [ -z "$files" ]; then
  echo "No agent files (AGENTS.md, CLAUDE.md, GEMINI.md, ...) in this repo."
  exit 0
fi
echo "$files" | while IFS= read -r path; do
  lines=$(wc -l < "$path" | tr -d ' ')
  heads=$(grep '^#' "$path" 2>/dev/null | sed 's/^#* *//' | head -4 | awk '{ printf "%s%s", sep, $0; sep = ", " }')
  echo "$path  ($lines lines${heads:+; headings: $heads})"
  if grep -q 'agent-kit' "$path" && ! git ls-files --error-unmatch -- "$path" >/dev/null 2>&1; then
    echo "  guess:    KIT: made by agent-kit on this computer"
    continue
  fi
  if git ls-files --error-unmatch -- "$path" >/dev/null 2>&1; then
    on_remote=$(git log --remotes -1 --format=%h -- "$path" 2>/dev/null)
    if [ -n "$on_remote" ]; then where="on the remote"; else where="only on this computer (never pushed)"; fi
    echo "  git:      tracked, $where"
    who=$(git log --format=%ae -- "$path" | sort | uniq -c | sort -rn | head -3 |
      awk -v me="$ME" '{ name = ($2 == me ? "you" : $2); printf "%s%s (%s)", sep, name, $1; sep = ", " }')
    echo "  authors:  $who"
    echo "  changed:  $(git log -1 --format=%cs -- "$path")"
    others=$(git log --format=%ae -- "$path" | sort -u | grep -cvxF "${ME:-<none>}")
    if [ -z "$on_remote" ] && [ "$others" -eq 0 ]; then
      echo "  guess:    YOURS: only you wrote it, and it was never pushed"
    elif [ -z "$on_remote" ]; then
      echo "  guess:    TEAM FILE: other people wrote it (not on the remote yet)"
    elif [ "$others" -eq 0 ]; then
      echo "  guess:    PROBABLY YOURS: on the remote, but only you wrote it"
    else
      echo "  guess:    TEAM FILE: other people wrote it and it is on the remote"
    fi
  else
    echo "  git:      not in git"
    echo "  guess:    YOURS: it exists only on this computer"
  fi
done
