#!/bin/sh
# Connect your local kit maps (AGENTS.agentk.md) to each coding agent, in a
# team repo, without touching the team's files. Beside mode only
# (MAPS="beside" in .agents/agent-kit.conf). Everything it writes stays on
# this computer: .git/info/exclude hides it, and check.sh blocks a commit.
#
# Usage: sh .agents/kit/wire.sh           write or refresh the wiring
#        sh .agents/kit/wire.sh --plan    show what it finds and would do; change nothing
#        sh .agents/kit/wire.sh --check   exit 1 when wiring is missing or stale
#        sh .agents/kit/wire.sh --remove  delete everything it made
#        sh .agents/kit/wire.sh --quiet   write or refresh, print nothing
#
# In each folder with your map or a team AGENTS.md (agents from AGENTS_USED):
#   Claude Code  CLAUDE.local.md, a marked block: @AGENTS.md (the team's,
#                when the folder has no CLAUDE.md; a CLAUDE.local.md stops
#                Claude reading AGENTS.md by itself), then @AGENTS.agentk.md.
#                Your own lines outside the block stay.
#   Codex        AGENTS.override.md: the team's AGENTS.md, then your map.
#                Codex reads it instead of AGENTS.md in that folder, so it
#                is rewritten whenever either file changes.
#   Cursor       .cursor/rules/agent-kit.mdc at the root.
#   Gemini       reported only; the agent adds your map to context.fileName
#                in ~/.gemini/settings.json with your yes.
# A file git tracks (the team's) is never written; it is reported instead.

MODE=write
case "${1:-}" in
  --plan) MODE=plan ;; --check) MODE=check ;; --remove) MODE=remove ;; --quiet) MODE=quiet ;;
  "") ;; *) echo "usage: wire.sh [--plan|--check|--remove|--quiet]"; exit 2 ;;
esac
cd "$(git rev-parse --show-toplevel)" || exit 1
MAPS="own"
DOC_SUFFIX=""
AGENTS_USED="claude codex"
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
[ -f .agents/agent-kit.conf ] && . ./.agents/agent-kit.conf
if [ "$MAPS" != beside ]; then
  [ "$MODE" = plan ] && echo "MAPS is \"$MAPS\": the kit owns AGENTS.md here, so there is nothing to wire"
  exit 0
fi
MAP_FILE="AGENTS${DOC_SUFFIX:-.agentk.md}"
BEGIN="<!-- agent-kit:begin (local only: written by .agents/kit/wire.sh, never committed) -->"
END="<!-- agent-kit:end -->"
MARK="agent-kit: written by .agents/kit/wire.sh"
TMP=$(mktemp -d) || exit 1
trap 'rm -rf "$TMP"' EXIT
STALE=0

uses() { case " $AGENTS_USED " in *" $1 "*) return 0 ;; esac; return 1; }
rel() { if [ "$1" = . ]; then printf '%s' "$2"; else printf '%s/%s' "$1" "$2"; fi; }
tracked() { git ls-files --error-unmatch -- "$1" >/dev/null 2>&1; }
say() {  # say <agent> <action> <path> <why>
  case "$MODE" in
    plan) printf '%-7s %-7s %-34s %s\n' "$1" "$2" "$3" "$4" ;;
    write) [ "$2" = ok ] || printf '%-7s %-7s %-34s %s\n' "$1" "$2" "$3" "$4" ;;
    check) case "$2" in write|remove) echo "$3: $4" ;; esac ;;
  esac
}
same() { [ -f "$1" ] && [ "$(cksum < "$1")" = "$(cksum < "$2")" ]; }  # same <file> <expected>
apply() {  # apply <agent> <path> <expected file> <why>: write it when it differs
  if same "$2" "$3"; then say "$1" ok "$2" "up to date"; return; fi
  STALE=1
  say "$1" write "$2" "$4"
  case "$MODE" in write|quiet) mkdir -p "$(dirname "$2")"; cp "$3" "$2" ;; esac
}
drop() {  # drop <agent> <path> <why>: delete a file the kit made
  STALE=1
  say "$1" remove "$2" "$3"
  case "$MODE" in write|quiet|remove) rm -f "$2" ;; esac
}

folders() {  # folders with your map or a team AGENTS.md, outside skipped folders
  set -- -name '.?*' -o -name '_*'
  for skip in $SKIP_DIRS $LEARNING_DIR; do set -- "$@" -o -name "$skip"; done
  find . -type d \( "$@" \) -prune -o -type f \( -name "$MAP_FILE" -o -name AGENTS.md \) -print |
    sed -e 's|^\./||' -e '/\//!s|.*|.|' -e 's|/[^/]*$||' | LC_ALL=C sort -u
}
made_here() {  # made_here: every file the kit made in an earlier run
  set -- -name '.?*' -o -name '_*'
  for skip in $SKIP_DIRS $LEARNING_DIR; do set -- "$@" -o -name "$skip"; done
  find . -type d \( "$@" \) -prune -o -type f \( -name CLAUDE.local.md -o -name AGENTS.override.md \) -print |
    sed 's|^\./||' | while IFS= read -r file; do grep -q 'agent-kit' "$file" && ! tracked "$file" && echo "$file"; done
}

claude_block() {  # claude_block <folder> <out>: the kit block for CLAUDE.local.md, empty when none is needed
  : > "$2"
  team=no; mine=no
  [ -f "$(rel "$1" AGENTS.md)" ] && [ ! -f "$(rel "$1" CLAUDE.md)" ] && [ ! -f "$(rel "$1" .claude/CLAUDE.md)" ] && team=yes
  [ -f "$(rel "$1" "$MAP_FILE")" ] && mine=yes
  [ "$team" = yes ] || [ "$mine" = yes ] || return 0
  {
    echo "$BEGIN"
    [ "$team" = yes ] && echo "@AGENTS.md"
    if [ "$mine" = yes ]; then
      echo "The team's rules (their CLAUDE.md and AGENTS.md) come first. My own map and rules add to them:"
      echo "@$MAP_FILE"
    fi
    echo "$END"
  } > "$2"
}
claude_file() {  # claude_file <path> <block> <out>: the file with its kit block replaced
  : > "$3"
  if [ -f "$1" ]; then
    awk -v b="$BEGIN" -v e="$END" '$0 == b { skip = 1 } !skip { print } $0 == e { skip = 0 }' "$1" > "$3"
  fi
  if [ -s "$2" ]; then
    [ -s "$3" ] && [ -n "$(tail -c 1 "$3")" ] && echo >> "$3"
    cat "$2" >> "$3"
  fi
}
codex_file() {  # codex_file <folder> <out>: the override Codex reads instead of AGENTS.md
  team=$(rel "$1" AGENTS.md); mine=$(rel "$1" "$MAP_FILE")
  : > "$2"
  [ -f "$mine" ] || return 0
  {
    echo "<!-- $MARK from AGENTS.md and $MAP_FILE in this folder; local only, never"
    echo "     committed. Do not edit it: edit those files, then run sh .agents/kit/wire.sh -->"
    if [ -f "$team" ]; then
      echo
      cat "$team"
      [ -n "$(tail -c 1 "$team")" ] && echo
      echo
      echo "---"
      echo
      echo "<!-- My own map and rules (local only). The team's rules above come first. -->"
    fi
    echo
    cat "$mine"
  } > "$2"
}

wire_folder() {  # wire_folder <folder>
  folder=$1
  if uses claude; then
    path=$(rel "$folder" CLAUDE.local.md)
    claude_block "$folder" "$TMP/block"
    if [ -f "$path" ] && tracked "$path"; then
      [ -s "$TMP/block" ] && say claude skip "$path" "tracked by git (the team's): Claude cannot load your map here; see ~/.claude/CLAUDE.md"
    else
      claude_file "$path" "$TMP/block" "$TMP/claude"
      if [ -s "$TMP/claude" ]; then
        apply claude "$path" "$TMP/claude" "imports $(grep '^@' "$TMP/block" | tr '\n' ' ')"
      elif [ -f "$path" ]; then
        drop claude "$path" "no map here any more"
      fi
    fi
  fi
  if uses codex; then
    path=$(rel "$folder" AGENTS.override.md)
    codex_file "$folder" "$TMP/codex"
    if [ -f "$path" ] && { tracked "$path" || ! grep -q "$MARK" "$path"; }; then
      [ -s "$TMP/codex" ] && say codex skip "$path" "not the kit's (tracked or hand-written): Codex cannot load your map here"
    elif [ -s "$TMP/codex" ]; then
      why="your map"; [ -f "$(rel "$folder" AGENTS.md)" ] && why="team AGENTS.md first, then your map"
      apply codex "$path" "$TMP/codex" "$why"
    elif [ -f "$path" ]; then
      drop codex "$path" "no map here any more"
    fi
  fi
}

if [ "$MODE" = remove ]; then
  made_here | while IFS= read -r file; do
    if [ "${file##*/}" = CLAUDE.local.md ]; then
      : > "$TMP/none"; claude_file "$file" "$TMP/none" "$TMP/rest"
      if [ -s "$TMP/rest" ] && grep -q '[^[:space:]]' "$TMP/rest"; then cp "$TMP/rest" "$file"; echo "removed the kit block from $file"
      else rm -f "$file"; echo "removed $file"; fi
    else
      rm -f "$file"; echo "removed $file"
    fi
  done
  [ -f .cursor/rules/agent-kit.mdc ] && ! tracked .cursor/rules/agent-kit.mdc && rm -f .cursor/rules/agent-kit.mdc &&
    echo "removed .cursor/rules/agent-kit.mdc"
  exit 0
fi

folders > "$TMP/folders"
while IFS= read -r folder <&3; do wire_folder "$folder"; done 3< "$TMP/folders"

# Wiring left behind in folders that no longer have a map.
made_here | while IFS= read -r file; do
  grep -qxF "$(dirname "$file")" "$TMP/folders" || echo "$file"
done > "$TMP/orphans"
while IFS= read -r file <&3; do
  if [ "${file##*/}" = CLAUDE.local.md ]; then
    : > "$TMP/none"; claude_file "$file" "$TMP/none" "$TMP/rest"
    if grep -q '[^[:space:]]' "$TMP/rest" 2>/dev/null; then apply claude "$file" "$TMP/rest" "no map here any more"
    else drop claude "$file" "no map here any more"; fi
  else
    drop codex "$file" "no map here any more"
  fi
done 3< "$TMP/orphans"

if uses cursor; then
  path=.cursor/rules/agent-kit.mdc
  {
    echo "---"
    echo "description: My agent kit maps (local only, never committed)"
    echo "alwaysApply: true"
    echo "---"
    echo "<!-- $MARK; local only, never committed -->"
    echo "Before you work in a folder, read the $MAP_FILE files on the path to it."
    echo "The team's AGENTS.md rules come first; mine add to them."
  } > "$TMP/mdc"
  if [ -f "$path" ] && tracked "$path"; then say cursor skip "$path" "tracked by git"
  else apply cursor "$path" "$TMP/mdc" "always-on rule that points to your maps"; fi
fi

if uses gemini || uses antigravity; then
  if tracked .gemini/settings.json && grep -q '"fileName"' .gemini/settings.json; then
    say gemini limit ".gemini/settings.json" "the team's context.fileName wins: Gemini cannot load $MAP_FILE here; use the ~/.gemini/GEMINI.md note"
  elif grep -qs "$MAP_FILE" "$HOME/.gemini/settings.json"; then
    say gemini ok "~/.gemini/settings.json" "context.fileName includes $MAP_FILE"
  else
    say gemini todo "~/.gemini/settings.json" "add $MAP_FILE to context.fileName (the agent does it with your yes)"
  fi
fi

[ "$MODE" = check ] && [ "$STALE" = 1 ] && exit 1
exit 0
