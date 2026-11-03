#!/bin/sh
# Task files: notes for one piece of work in progress, kept on disk so a
# fresh session (or another agent) continues from the file, not from the
# chat. They live in .agents/work/<name><DOC_SUFFIX> and are never committed.
#
# Usage: sh .agents/kit/task.sh new <name>     create one from the template; prints its path
#        sh .agents/kit/task.sh list           open tasks: name, status, branch, next step (tab-separated)
#        sh .agents/kit/task.sh path <name>    print its path
#        sh .agents/kit/task.sh close <name>   delete it (wrap-up runs this after it moved the lasting facts)
#
# A name is lowercase letters, digits and dashes: fix-login-timeout.
# Installed by agent_dev_kit; do not edit here.

KIT_DIR=$(cd "$(dirname "$0")" && pwd)  # before cd: $0 may be relative
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "task: not inside a git repository"; exit 1; }
cd "$ROOT" || exit 1
DOC_SUFFIX=".agentk.md"
. "$KIT_DIR/lib.sh" && load_conf
SUFFIX=${DOC_SUFFIX:-.md}
WORK=.agents/work

slug() {  # slug <words>: a safe file name
  printf '%s' "$*" | tr 'A-Z_ ' 'a-z--' | sed 's/[^a-z0-9-]//g; s/--*/-/g; s/^-//; s/-$//'
}
task_path() {
  name=$(slug "$1")
  [ -n "$name" ] || { echo "task: give a name, such as: fix-login-timeout"; exit 2; }
  printf '%s/%s%s' "$WORK" "$name" "$SUFFIX"
}
field() {  # field <file> <name>: the value of a "Name: value" header line
  awk -v f="$2" '{ sub(/\r$/, "") } index($0, f ": ") == 1 { print substr($0, length(f) + 3); exit } /^## / { exit }' "$1"
}
next_step() {  # next_step <file>: the first line of text under "## Next"
  awk '{ sub(/\r$/, "") }
       /^## / { inside = ($0 ~ /^## Next[ \t]*$/); next }
       inside && /^<!--/ { comment = 1 }
       inside && comment { if ($0 ~ /-->/) comment = 0; next }
       inside && NF { sub(/^[-*] /, ""); print; exit }' "$1"
}

case "${1:-}" in
  new)
    shift
    file=$(task_path "$*") || { echo "$file"; exit 2; }
    if [ -e "$file" ]; then echo "task: $file exists; continue it, or choose another name"; exit 1; fi
    template="$KIT_DIR/task.agentk.md"
    [ -f "$template" ] || { echo "task: $template is missing; run agent_dev_kit/install.sh"; exit 1; }
    mkdir -p "$WORK" || exit 1
    branch=$(git symbolic-ref --short -q HEAD || echo "(detached)")
    title=$(printf '%s' "$*" | sed 's/[&|\\]/\\&/g')
    branch_text=$(printf '%s' "$branch" | sed 's/[&|\\]/\\&/g')
    sed -e "s|<title>|$title|" -e "s|<branch>|$branch_text|" -e "s|<date>|$(date +%Y-%m-%d)|" "$template" > "$file" || exit 1
    git check-ignore -q "$file" ||
      echo "task: WARNING $WORK/ is not git-ignored here; run agent_dev_kit/install.sh so a task file is never committed"
    echo "$file"
    ;;
  list)
    for file in "$WORK"/*"$SUFFIX"; do
      [ -f "$file" ] || continue
      name=${file##*/}; name=${name%"$SUFFIX"}
      printf '%s\t%s\t%s\t%s\n' "$name" "$(field "$file" Status)" "$(field "$file" Branch)" "$(next_step "$file")"
    done
    ;;
  path)
    shift
    task_path "$*" && echo
    ;;
  close)
    shift
    file=$(task_path "$*") || { echo "$file"; exit 2; }
    [ -f "$file" ] || { echo "task: no task file $file"; exit 1; }
    rm -f "$file" && echo "task: closed $file"
    rmdir "$WORK" 2>/dev/null
    exit 0
    ;;
  *)
    echo "usage: task.sh new <name> | list | path <name> | close <name>"; exit 2 ;;
esac
