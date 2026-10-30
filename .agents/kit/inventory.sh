#!/bin/sh
# House-cleaning inventory: list every agent and doc file in the repo,
# grouped, with what git knows about each, so the agent can ask the user
# what to keep, merge, mark as the repo's own, or remove.
#
# Usage: sh inventory.sh [repo]   (read only; it changes nothing)
# Works before the kit is installed (run it from agent_dev_kit/templates/kit/)
# and after (.agents/kit/inventory.sh). Needs sh, git, find, awk, cksum.

ROOT=$(git -C "${1:-.}" rev-parse --show-toplevel 2>/dev/null) || {
  echo "inventory: not inside a git repository"; exit 1; }
cd "$ROOT" || exit 1

SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
DOC_SUFFIX=""
OWN_SKILLS=""
[ -f .agents/agent-kit.conf ] && . ./.agents/agent-kit.conf
KIT_SKILLS="lineage scaffold docs-sync learning-docs catch-up wrap-up explain-diff"
TODAY=$(date +%s)
SEEN=$(mktemp) || exit 1
trap 'rm -f "$SEEN"' EXIT

find_files() {  # find_files <find tests...>: files outside .git and skipped folders
  prune="-name .git"
  for skip in $SKIP_DIRS; do prune="$prune -o -name $skip"; done
  # shellcheck disable=SC2086  # the prune and test expressions are word lists
  find . \( $prune \) -prune -o -type f \( "$@" \) -print 2>/dev/null | sed 's|^\./||' | LC_ALL=C sort
}

describe() {  # describe <path>: state, last change, size, and notes
  path=$1
  if git ls-files --error-unmatch -- "$path" >/dev/null 2>&1; then state="tracked"
  elif git check-ignore -q -- "$path"; then state="ignored"
  else state="untracked"; fi
  day=$(git log -1 --format=%cs -- "$path" 2>/dev/null)
  stamp=$(git log -1 --format=%ct -- "$path" 2>/dev/null)
  if [ -d "$path" ]; then size="$(find "$path" -type f | wc -l | tr -d ' ') files"
  else size="$(wc -l < "$path" | tr -d ' ') lines"; fi
  note=""
  if [ -n "$stamp" ] && [ $(( (TODAY - stamp) / 86400 )) -gt 180 ]; then
    note="not changed in $(( (TODAY - stamp) / 86400 )) days"
  fi
  [ "$state" = tracked ] && [ -z "$day" ] && state="staged"
  # Pointer files are identical on purpose; do not call them duplicates.
  if [ -f "$path" ] && [ "$(cat "$path")" != "@AGENTS.md" ]; then
    sum=$(cksum < "$path")
    first=$(awk -v s="$sum" -F '\t' '$1 == s { print $2; exit }' "$SEEN")
    if [ -n "$first" ]; then note="${note:+$note; }same content as $first"
    else printf '%s\t%s\n' "$sum" "$path" >> "$SEEN"; fi
  fi
  printf '  %s  [%s, %s, %s]%s\n' "$path" "$state" "${day:-never committed}" "$size" "${note:+  -- $note}"
}

group() {  # group <title> <paths...>: print a group when it has entries
  title=$1; shift
  [ $# -gt 0 ] || return
  echo "$title"
  for path in "$@"; do describe "$path"; done
  echo
}

list_git_hooks() {  # hooks installed in .git/hooks (not versioned), samples left out
  dir=$(git rev-parse --git-path hooks)
  for hook in "$dir"/*; do
    [ -f "$hook" ] || continue
    case "$hook" in
      *.sample) ;;
      *) basename "$hook" ;;
    esac
  done
}

skill_note() {  # skill_note <skill folder>
  name=$(basename "$1")
  case " $OWN_SKILLS " in *" $name "*) echo "repo-owned (OWN_SKILLS)"; return ;; esac
  case " $KIT_SKILLS " in *" $name "*) echo "same name as a kit skill" ;; esac
}

echo "Agent and doc files in $ROOT"
echo "(state, last commit, size; read only)"
echo

# shellcheck disable=SC2046  # file lists are one path per line, no spaces expected
if [ -n "$DOC_SUFFIX" ]; then
  # shellcheck disable=SC2046
  group "Kit docs: maps, pointer files and *$DOC_SUFFIX" $(find_files -name AGENTS.md -o -name CLAUDE.md -o -name GEMINI.md \
    -o -name "*$DOC_SUFFIX" | grep -v -E "^(\.agents|${LEARNING_DIR:-__none__})/")
else
  # shellcheck disable=SC2046
  group "Maps and pointer files" $(find_files -name AGENTS.md -o -name CLAUDE.md -o -name GEMINI.md)
fi
group "Rules for other agents" $(find_files -name .cursorrules -o -name .windsurfrules -o -name .clinerules \
  -o -path './.cursor/rules/*' -o -path './.github/copilot-instructions.md' -o -path './.github/instructions/*')

skills=$(for root in .agents/skills .claude/skills .codex/skills .cursor/skills .agent/skills .gemini/skills; do
  [ -d "$root" ] || continue
  [ -L "$root" ] && continue  # a link to another skills folder is listed there
  for skill in "$root"/*/; do [ -f "${skill}SKILL.md" ] && echo "${skill%/}"; done
done)
if [ -n "$skills" ]; then
  echo "Skills"
  for skill in $skills; do
    line=$(describe "$skill")
    extra=$(skill_note "$skill")
    printf '%s%s\n' "$line" "${extra:+  -- $extra}"
  done
  [ -L .claude/skills ] && echo "  .claude/skills -> $(readlink .claude/skills) (link, not a copy)"
  echo
fi

group "Commands and prompts" $(find_files -path './.claude/commands/*' -o -path './.cursor/commands/*' -o -path './.codex/prompts/*')
hooks=$(find_files -path './.claude/settings*.json' -o -path './.codex/hooks.json' -o -path './.codex/config.toml' \
  -o -path './.githooks/*' -o -path './.husky/*' -o -name .pre-commit-config.yaml)
git_hooks=$(list_git_hooks)
group "Hooks and agent settings" $hooks
[ -n "$git_hooks" ] && echo "  (installed in .git/hooks, not versioned: $(echo $git_hooks | tr '\n' ' '))" && echo
group "Decisions, plans and ledgers" $(find_files -name DECISIONS.md -o -name PLAN.md -o -name EXPERIMENTS.md \
  -o -path './docs/adr/*' -o -path './adr/*' -o -path './doc/adr/*')
[ -n "$LEARNING_DIR" ] && [ -d "$LEARNING_DIR" ] && group "Learning docs" $(find_files -path "./$LEARNING_DIR/*")

others=$(find_files -name '*.md' | grep -v -E '(^|/)(AGENTS|CLAUDE|GEMINI|DECISIONS|PLAN|EXPERIMENTS|SKILL)\.md$' |
  grep -v -F "${DOC_SUFFIX:-__no_suffix__}" |
  grep -v -E "^(\.agents|\.claude|\.codex|\.cursor|\.github|${LEARNING_DIR:-__none__})/")
others=$(for path in $others; do git check-ignore -q -- "$path" || echo "$path"; done)
group "Other Markdown" $others
