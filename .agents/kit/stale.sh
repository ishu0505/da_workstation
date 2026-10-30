#!/bin/sh
# Docs that may be stale: each doc whose anchors changed since the doc was
# last updated. Read only; docs-sync, wrap-up and catch-up start from it.
#
# Usage: sh .agents/kit/stale.sh [since <ref>]
# Prints one line per doc: <doc> TAB <what changed> TAB <kit|yours>.
# "kit" docs (maps, pointers, files ending in DOC_SUFFIX) the agent may fix;
# "yours" docs (every other README) it only reports to the people here.
#
# Anchors are the code a doc describes:
#   AGENTS.md      the paths it names; a named folder counts only when a file
#                  in it is added, removed or renamed
#   README.md      <!-- agent-kit: source_paths a/ b.py --> (paths from its
#                  folder), else its folder; the root README needs the comment
#   learning docs  source_paths and source_commit in the front matter
#   decisions      source_paths in the front matter of accepted decisions
# "since <ref>" compares every doc with the merge base of <ref> instead.
cd "$(git rev-parse --show-toplevel)" || exit 1
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
DECISIONS_DIR=""
DOC_SUFFIX=""
POINTER_FILES="CLAUDE.md"
[ -f .agents/agent-kit.conf ] && . ./.agents/agent-kit.conf
KIT_MD=${DOC_SUFFIX:-.md}
if [ "${MAPS:-own}" = beside ]; then MAP_FILE="AGENTS${DOC_SUFFIX:-.agentk.md}"; else MAP_FILE="AGENTS.md"; fi
TAB=$(printf '\t')
SINCE=""
if [ "${1:-}" = since ]; then
  SINCE=$(git merge-base "${2:-HEAD}" HEAD) || { echo "stale.sh: no common commit with ${2:-}"; exit 1; }
fi

base_of() {  # base_of <doc> [source_commit]: the commit to compare with
  if [ -n "$SINCE" ]; then echo "$SINCE"; return; fi
  if [ -n "${2:-}" ] && git cat-file -e "$2^{commit}" 2>/dev/null; then echo "$2"; return; fi
  git log -1 --format=%H -- "$1"
}
changes() {  # changes <base> <all|structure> <path>...: changed non-Markdown paths
  base=$1; kind=$2; shift 2
  [ -n "$base" ] && [ $# -gt 0 ] || return 0
  { git -c core.quotepath=off diff --name-status "$base" -- "$@"
    git -c core.quotepath=off ls-files --others --exclude-standard -- "$@" | sed "s/^/A$TAB/"
  } | awk -F "$TAB" -v kind="$kind" '(kind == "all" || $1 ~ /^[ADR]/) && $NF !~ /\.md$/ { print $NF }' | LC_ALL=C sort -u
}
owner() {  # owner <doc>: kit (the kit wrote it) or yours
  base=${1##*/}
  if [ "${MAPS:-own}" = beside ]; then  # the team's agent files are theirs
    case "$base" in AGENTS.md|CLAUDE.md|GEMINI.md) echo yours; return ;; esac
  fi
  case " AGENTS.md $POINTER_FILES " in *" $base "*) echo kit; return ;; esac
  if [ -n "$DOC_SUFFIX" ]; then
    case "$base" in *"$DOC_SUFFIX") echo kit ;; *) echo yours ;; esac
  else
    echo kit  # kit 0.5 and older: unit READMEs were the kit's own
  fi
}
say() {  # say <doc> <changed paths>
  [ -n "$2" ] || return 0
  count=$(printf '%s\n' "$2" | wc -l | tr -d ' ')
  shown=$(printf '%s\n' "$2" | head -3 | tr '\n' ' ')
  [ "$count" -gt 3 ] && shown="$shown(+$((count - 3)) more)"
  printf '%s\t%s changed: %s\t%s\n' "$1" "$count" "$shown" "$(owner "$1")"
}
front_paths() {  # front_paths <doc> <key>: the list in "key: [a, b]" of the front matter
  awk -v key="$2" 'NR == 1 && $0 != "---" { exit } NR > 1 && $0 == "---" { exit }
    index($0, key ":") == 1 { sub(/^[^:]*:[ \t]*\[?/, ""); sub(/\][ \t\r]*$/, ""); gsub(/[,"\047]/, " "); print }' "$1"
}
find_docs() {  # find_docs <name pattern>: matching files outside skipped folders
  pattern=$1
  set -- -name '.?*' -o -name '_*'
  for skip in $SKIP_DIRS $LEARNING_DIR; do set -- "$@" -o -name "$skip"; done
  find . -type d \( "$@" \) -prune -o -type f -name "$pattern" -print | sed 's|^\./||' | LC_ALL=C sort
}

# Maps.
for map in $(find_docs "$MAP_FILE"); do
  base=$(base_of "$map"); [ -n "$base" ] || continue
  anchors=$(sh .agents/kit/check.sh --anchors "$map" | cut -f3 | LC_ALL=C sort -u)
  files=""; folders=""
  for path in $anchors; do
    if [ -d "$path" ]; then folders="$folders $path"; else files="$files $path"; fi
  done
  # shellcheck disable=SC2086  # path lists
  team=""
  if [ "${MAPS:-own}" = beside ]; then  # the team's rules in this folder changed: align with them
    folder=$(dirname "$map")
    for doc in AGENTS.md CLAUDE.md GEMINI.md; do
      [ "$folder" = . ] && path=$doc || path="$folder/$doc"
      [ -f "$path" ] && git -c core.quotepath=off diff --name-only "$base" -- "$path" | grep -q . && team="$team
$path (team rules)"
    done
  fi
  say "$map" "$( { changes "$base" all $files; changes "$base" structure $folders; printf '%s\n' "$team" | sed '/^$/d'; } | LC_ALL=C sort -u)"
done

# READMEs: the kit's own (README.agentk.md) and the people's (README.md).
readmes=$(find_docs README.md)
[ -n "$DOC_SUFFIX" ] && readmes="$readmes $(find_docs "README$DOC_SUFFIX")"
for readme in $readmes; do
  base=$(base_of "$readme"); [ -n "$base" ] || continue
  folder=$(dirname "$readme")
  anchors=$(sed -n 's/.*<!-- agent-kit: source_paths \(.*\) -->.*/\1/p' "$readme")
  if [ -n "$anchors" ]; then
    paths=""; for path in $anchors; do
      if [ "$folder" = . ]; then paths="$paths $path"; else paths="$paths $folder/$path"; fi
    done
  elif [ "$folder" != . ]; then paths=$folder
  else continue
  fi
  # shellcheck disable=SC2086
  say "$readme" "$(changes "$base" all $paths)"
done

# Learning docs.
if [ -n "$LEARNING_DIR" ] && [ -d "$LEARNING_DIR" ]; then
  for doc in $(find "$LEARNING_DIR" -name '*.md' -type f | LC_ALL=C sort); do
    paths=$(front_paths "$doc" source_paths); [ -n "$paths" ] || continue
    base=$(base_of "$doc" "$(front_paths "$doc" source_commit | tr -d ' ')"); [ -n "$base" ] || continue
    # shellcheck disable=SC2086
    say "$doc" "$(changes "$base" all $paths)"
  done
fi

# Accepted decisions.
if [ -n "$DECISIONS_DIR" ] && [ -d "$DECISIONS_DIR" ]; then
  for doc in "${DECISIONS_DIR%/}"/D-*.md; do
    [ -f "$doc" ] || continue
    grep -q '^status:[[:blank:]]*accepted' "$doc" || continue
    paths=$(front_paths "$doc" source_paths); [ -n "$paths" ] || continue
    base=$(base_of "$doc"); [ -n "$base" ] || continue
    # shellcheck disable=SC2086
    say "$doc" "$(changes "$base" all $paths)"
  done
fi
