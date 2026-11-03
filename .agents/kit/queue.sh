#!/bin/sh
# The catch-up queue: commits that changed code without the docs covering it
# (usually code written by hand). Kept in .git/agent-kit/pending, which is
# never committed.
#
# Usage: sh .agents/kit/queue.sh add [<commit>]   queue it when it needs docs (post-commit)
#        sh .agents/kit/queue.sh list             commit, date, files and subject, one per line
#        sh .agents/kit/queue.sh done <commit>... remove commits the catch-up skill handled
#        sh .agents/kit/queue.sh count
#
# A commit needs docs when it changes a code file (CODE_EXTENSIONS) and none
# of that file's covering docs: the nearest AGENTS.md, kit detail doc
# (README.agentk.md) or the people's README.md above it.
# A commit message with "[no-docs]" is never queued.
KIT_LIB="$(cd "$(dirname "$0")" && pwd)/lib.sh"  # before cd: $0 may be relative
cd "$(git rev-parse --show-toplevel)" || exit 1
CODE_EXTENSIONS="py js ts tsx jsx go rs c cc cpp h hpp java kt cs rb php swift sql sh"
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
DOC_SUFFIX=""
. "$KIT_LIB" && load_conf
UNIT_DOC="README${DOC_SUFFIX:-.md}"
if [ "${MAPS:-own}" = beside ]; then MAP_FILE="AGENTS${DOC_SUFFIX:-.agentk.md}"; else MAP_FILE="AGENTS.md"; fi
DIR="$(git rev-parse --git-common-dir)/agent-kit"
PENDING="$DIR/pending"

covering_docs() {  # covering_docs <path>: the docs in the nearest folder above it that has any, one per line
  dir=$(dirname "$1")
  while :; do
    found=""
    for doc in AGENTS.md "$MAP_FILE" "$UNIT_DOC" README.md; do
      path=$doc; [ "$dir" = . ] || path="$dir/$doc"
      [ -f "$path" ] && found="$found$path
"
    done
    if [ -n "$found" ] || [ "$dir" = . ]; then printf '%s' "$found"; return; fi
    dir=$(dirname "$dir")
  done
}

case "${1:-list}" in
  add)
    sha=$(git rev-parse --verify -q "${2:-HEAD}^{commit}") || exit 0
    git log -1 --format=%B "$sha" | grep -qF '[no-docs]' && exit 0
    [ -f "$PENDING" ] && grep -q "^$sha " "$PENDING" && exit 0
    files=$(git -c core.quotepath=off diff-tree --root --no-commit-id --name-only -r "$sha")
    undocumented=0
    while IFS= read -r file; do
      [ -n "$file" ] && is_code "$file" || continue
      docs=$(covering_docs "$file")
      touched=no
      while IFS= read -r doc; do
        [ -n "$doc" ] && printf '%s\n' "$files" | grep -qxF -- "$doc" && touched=yes
      done <<DOCS
$docs
DOCS
      [ "$touched" = no ] && undocumented=$((undocumented + 1))
    done <<FILES
$files
FILES
    [ "$undocumented" -gt 0 ] || exit 0
    mkdir -p "$DIR"
    printf '%s %s %s-files %s\n' "$sha" "$(git log -1 --format=%cs "$sha")" "$undocumented" \
      "$(git log -1 --format=%s "$sha")" >> "$PENDING"
    ;;
  list)
    [ -s "$PENDING" ] || exit 0
    # Drop commits that no longer exist (amended or rebased away).
    while IFS= read -r line; do
      git cat-file -e "${line%% *}^{commit}" 2>/dev/null && printf '%s\n' "$line"
    done < "$PENDING" > "$PENDING.tmp" && mv "$PENDING.tmp" "$PENDING"
    cat "$PENDING"
    ;;
  done)
    shift
    [ -f "$PENDING" ] || exit 0
    for rev in "$@"; do
      sha=$(git rev-parse --verify -q "$rev^{commit}") || sha=$rev
      grep -v "^$sha " "$PENDING" > "$PENDING.tmp"; mv "$PENDING.tmp" "$PENDING"
    done
    ;;
  count)
    if [ -f "$PENDING" ]; then wc -l < "$PENDING" | tr -d ' '; else echo 0; fi
    ;;
  *) echo "usage: queue.sh add [<commit>] | list | done <commit>... | count"; exit 2 ;;
esac
