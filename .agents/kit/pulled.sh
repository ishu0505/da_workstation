#!/bin/sh
# Team pulls: commits from other people that changed code since the docs
# were last checked on this branch. The catch-up queue sees only your own
# commits; this sees what a pull, merge, rebase or checkout brought in, so
# docs-sync can bring the docs and learning docs up to date.
#
# The mark "docs checked up to <commit>" is kept per branch in
# .git/agent-kit/synced/, which is never committed or pushed.
#
# Usage: sh .agents/kit/pulled.sh status       what arrived, or nothing
#        sh .agents/kit/pulled.sh count        the number of those commits
#        sh .agents/kit/pulled.sh base         the mark, for: docs-sync since <base>
#        sh .agents/kit/pulled.sh done [<rev>] the docs are checked up to <rev> (default HEAD)
#
# On a branch without a mark, the mark is set to HEAD and nothing is
# reported: the kit starts counting from now. When nothing from other
# people arrived, the mark moves to HEAD by itself. A commit counts when its
# author email differs from git config user.email and it changes a code file
# (CODE_EXTENSIONS). Merge commits do not count; the commits they bring do.
KIT_LIB="$(cd "$(dirname "$0")" && pwd)/lib.sh"  # before cd: $0 may be relative
cd "$(git rev-parse --show-toplevel)" || exit 1
CODE_EXTENSIONS="py js ts tsx jsx go rs c cc cpp h hpp java kt cs rb php swift sql sh"
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
. "$KIT_LIB" && load_conf
DIR="$(git rev-parse --git-common-dir)/agent-kit/synced"
MAX=500  # look at most at this many commits

branch=$(git symbolic-ref -q --short HEAD) || exit 0  # detached HEAD: nothing to say
head=$(git rev-parse -q --verify HEAD) || exit 0      # no commit yet
MARK="$DIR/$(printf '%s' "$branch" | sed 's|/|__|g')"

set_mark() { mkdir -p "$DIR" && printf '%s\n' "$1" > "$MARK"; }

base() {  # base: the mark, made valid (set to HEAD when missing or gone)
  mark=$(cat "$MARK" 2>/dev/null)
  if [ -z "$mark" ] || ! git cat-file -e "$mark^{commit}" 2>/dev/null; then
    set_mark "$head"; echo "$head"; return
  fi
  # A rebase or a reset can take the mark off this branch: use the fork point.
  git merge-base --is-ancestor "$mark" "$head" 2>/dev/null || mark=$(git merge-base "$mark" "$head" 2>/dev/null) || mark=$head
  echo "$mark"
}

arrived() {  # arrived <base>: "<sha> <author> <subject>" for each teammate commit that changed code
  me=$(git config user.email || echo "<none>")
  git log --no-merges -n "$MAX" --format='%H%x09%ae%x09%an%x09%s' "$1..$head" |
  while IFS="$(printf '\t')" read -r sha email name subject; do
    [ "$email" = "$me" ] && continue
    code=no
    while IFS= read -r file; do
      [ -n "$file" ] && is_code "$file" && { code=yes; break; }
    done <<FILES
$(git -c core.quotepath=off diff-tree --no-commit-id --name-only -r "$sha")
FILES
    [ "$code" = yes ] && printf '%s %s %s\n' "$(printf '%s' "$sha" | cut -c1-9)" "$name" "$subject"
  done
}

case "${1:-status}" in
  status|count)
    b=$(base)
    [ "$b" = "$head" ] && { [ "$1" = count ] && echo 0; exit 0; }
    list=$(arrived "$b")
    if [ -z "$list" ]; then
      set_mark "$head"  # only your own commits, or no code: nothing to check
      [ "$1" = count ] && echo 0
      exit 0
    fi
    n=$(printf '%s\n' "$list" | wc -l | tr -d ' ')
    if [ "$1" = count ]; then echo "$n"; exit 0; fi
    echo "$n commit(s) from other people changed code since the docs were last checked (since $(printf '%s' "$b" | cut -c1-9)):"
    printf '%s\n' "$list" | head -5 | sed 's/^/  /'
    [ "$n" -le 5 ] || echo "  and $((n - 5)) more"
    ;;
  base) base ;;
  done)
    rev=$(git rev-parse -q --verify "${2:-HEAD}^{commit}") || { echo "pulled.sh: no commit ${2:-HEAD}"; exit 1; }
    set_mark "$rev"
    ;;
  *) echo "usage: pulled.sh status | count | base | done [<rev>]"; exit 2 ;;
esac
