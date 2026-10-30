#!/bin/sh
# Agent kit checks: the agent maps stay short and true, and the setup
# around them (skills, hooks, git) works on every clone.
#
# Usage: .agents/kit/check.sh [--quiet] [--ci] [--run-commands]
#                             [--staged | --since <ref> | --changed <path>...]
#        .agents/kit/check.sh --anchors <map>...
#
#   no scope        every problem fails (by hand, and CI on a push)
#   --staged        pre-commit: only problems this commit causes fail; maps
#                   it cannot affect are not read (fast in a big repo)
#   --since <ref>   CI on a pull request: only problems the branch causes fail
#   --changed ...   the Stop hook: only problems these paths cause fail
#   --ci            skip the per-clone check (is the pre-commit hook active)
#   --run-commands  also run the ## Run and ## Check blocks of the maps in
#                   RUN_COMMANDS_MAPS
#   --anchors       print map, line and path for every path the maps name
#
# Reads .agents/agent-kit.conf. Prints each failure with its fix and exits 1
# when anything fails. Needs only sh, git, grep, awk and sed, so it works in a
# repo of any language. Installed by agent_dev_kit; do not edit in a target
# repo (install.sh overwrites it).

QUIET=0
CI=0
RUN_COMMANDS=0
ANCHORS=0
SCOPE=""
SINCE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --quiet) QUIET=1 ;;
    --ci) CI=1 ;;
    --run-commands) RUN_COMMANDS=1 ;;
    --staged) SCOPE=staged ;;
    --since) SCOPE=since; SINCE=${2:-}; shift ;;
    --changed) SCOPE=changed; shift; break ;;
    --anchors) ANCHORS=1; shift; break ;;
    *) echo "agent-kit: unknown option $1 (see the usage at the top of check.sh)"; exit 2 ;;
  esac
  shift
done

ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "agent-kit: not inside a git repository"; exit 1; }
cd "$ROOT" || exit 1

# Defaults; the repo's config overrides them. Thresholds left empty mean
# "every unit needs a map", which is how kit 0.4 and older behaved.
UNITS=""
UNIT_PARENTS=""
MAP_MIN_FILES=""
MAP_MIN_LINES=""
MAP_REQUIRED=""
MAP_EXEMPT=""
SKILLS_LINK="symlink"
SHARE="agent"
TOP_MAP_MAX=70
UNIT_MAP_MAX=40
POINTER_FILES="CLAUDE.md"
CODE_EXTENSIONS="py js ts tsx jsx go rs c cc cpp h hpp java kt cs rb php swift sql sh"
CHECK_ENV=""
RUN_COMMANDS_MAPS=""
LEDGER=""
DECISIONS_DIR=""
DOC_SUFFIX=""
MAPS="own"
AGENTS_USED="claude codex"
LEARNING_DIR="learning_docs"
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
CONF=".agents/agent-kit.conf"
if [ -f "$CONF" ]; then
  . "./$CONF"
else
  echo "FAIL $CONF is missing"
  echo "     fix: run agent_dev_kit/install.sh on this repo"
  exit 1
fi
# Every Markdown file the kit writes ends in DOC_SUFFIX (".agentk.md");
# empty means the plain names of kit 0.5 and older (README.md, DECISIONS.md).
KIT_MD=${DOC_SUFFIX:-.md}
UNIT_DOC="README$KIT_MD"
DECISIONS_INDEX="DECISIONS$KIT_MD"
PLAN_DOC="PLAN$KIT_MD"
# The kit's map file. MAPS="beside" (a team repo with its own AGENTS.md or
# CLAUDE.md): the kit's maps are AGENTS.agentk.md, kept on this computer
# only, and every AGENTS.md / CLAUDE.md / GEMINI.md is the team's.
if [ "$MAPS" = beside ]; then MAP_FILE="AGENTS${DOC_SUFFIX:-.agentk.md}"; else MAP_FILE="AGENTS.md"; fi
# File extensions that make a word in backticks look like a path.
DOC_EXTENSIONS="md txt json yaml yml toml lock cfg ini conf env html css scss xml csv proto sql sh ps1 bat mk gradle"
TAB=$(printf '\t')

FAILS=0
WARNS=0
MAP_COUNT=0
fail() {
  FAILS=$((FAILS + 1))
  printf 'FAIL %s\n     fix: %s\n' "$1" "$2"
}
report() {  # report <yes|no: caused by this change> <problem> <fix>
  if [ "$1" = yes ]; then fail "$2" "$3"; return; fi
  WARNS=$((WARNS + 1))
  [ "$QUIET" = 1 ] || printf 'WARN %s (not part of this change)\n     fix: %s\n' "$2" "$3"
}
yes_if() { if "$@"; then echo yes; else echo no; fi; }  # yes_if <test...>

# --- What this change touched -----------------------------------------------
CHANGED=""
GONE=""  # deleted or renamed away
removed_paths() { awk -F "$TAB" '$1 ~ /^[DR]/ { print $2 }'; }
case "$SCOPE" in
  staged)
    CHANGED=$(git -c core.quotepath=off diff --cached --name-only)
    GONE=$(git -c core.quotepath=off diff --cached --name-status -M | removed_paths) ;;
  since)
    base=$(git merge-base "$SINCE" HEAD 2>/dev/null) || {
      echo "FAIL --since $SINCE: no common commit with HEAD"
      echo "     fix: fetch the base branch (actions/checkout with fetch-depth: 0) or pass another ref"
      exit 1; }
    CHANGED=$( { git -c core.quotepath=off diff --name-only "$base"; git -c core.quotepath=off ls-files --others --exclude-standard; } )
    GONE=$(git -c core.quotepath=off diff --name-status -M "$base" | removed_paths) ;;
  changed)
    CHANGED=$(printf '%s\n' "$@")
    GONE=$(for path in "$@"; do [ -e "$path" ] || printf '%s\n' "$path"; done) ;;
esac

is_changed() {  # is_changed <path>: this change touched the file
  [ -z "$SCOPE" ] && return 0
  printf '%s\n' "$CHANGED" | grep -qxF -- "$1"
}
changed_under() {  # changed_under <folder>: this change touched a file inside
  [ -z "$SCOPE" ] && return 0
  printf '%s\n' "$CHANGED" | awk -v p="$1/" 'index($0, p) == 1 || p == "./" && NF { found = 1 } END { exit !found }'
}
was_removed() {  # was_removed <path>: this change deleted it, or a file inside it
  [ -n "$GONE" ] || return 1
  printf '%s\n' "$GONE" | awk -v p="$1" '$0 == p || index($0, p "/") == 1 { found = 1 } END { exit !found }'
}

# --- Folders and maps -----------------------------------------------------------
is_skipped() {  # is_skipped <folder name>
  case "$1" in .*|_*) return 0 ;; esac
  [ "$1" = "$LEARNING_DIR" ] && return 0
  for skip in $SKIP_DIRS; do [ "$1" = "$skip" ] && return 0; done
  return 1
}
rel() {  # rel <folder> <path>: the path from the repo root
  if [ "$1" = . ]; then printf '%s' "$2"; else printf '%s/%s' "$1" "$2"; fi
}
exists() {  # exists <folder> <path or glob>
  case "$2" in
    *[*?[]*) set -- "$1"/$2; [ -e "$1" ] ;;  # a glob (src/*View.svelte); unquoted on purpose
    *) [ -e "$1/$2" ] ;;
  esac
}

find_maps() {  # every kit map (MAP_FILE) outside skipped folders, as a path from the root
  set -- -name '.?*' -o -name '_*'
  for skip in $SKIP_DIRS $LEARNING_DIR; do set -- "$@" -o -name "$skip"; done
  find . -type d \( "$@" \) -prune -o -type f -name "$MAP_FILE" -print | sed 's|^\./||' | LC_ALL=C sort
}

unit_folders() {  # every unit: listed in UNITS, or a child of a UNIT_PARENTS folder
  for unit in $UNITS; do [ -d "$unit" ] && echo "${unit%/}"; done
  for parent in $UNIT_PARENTS; do
    [ -d "$parent" ] || continue
    for child in "${parent%/}"/*/; do
      [ -d "$child" ] || continue
      child=${child%/}
      is_skipped "${child##*/}" || echo "$child"
    done
  done
}

map_tokens() {  # map_tokens <map>: kind, line and text of every `word` in backticks
  # Kind S: a mapped path (table "Path" and "Read first" columns, the
  # "Further reading" list); it must exist. Kind P: any other word; it is
  # checked only when it looks like a path. Code blocks and lines ending in
  # <!-- agent-kit: skip --> are left out.
  awk '
    { sub(/\r$/, "") }
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    function emit(kind, text) {
      while (match(text, /`[^`]+`/)) {
        print kind "\t" NR "\t" substr(text, RSTART + 1, RLENGTH - 2)
        text = substr(text, RSTART + RLENGTH)
      }
    }
    function first(text,   word, rest) {  # the first word as S, the rest as P
      if (!match(text, /`[^`]+`/)) { emit("P", text); return }
      word = substr(text, RSTART, RLENGTH); rest = substr(text, RSTART + RLENGTH)
      emit("S", word); emit("P", rest)
    }
    /^[ \t]*(```|~~~)/ { fence = !fence; next }
    fence { next }
    /<!-- agent-kit: skip -->/ { next }
    /^#/ { section = $0; header = 0; emit("P", $0); next }
    !/^\|/ { header = 0 }
    /^- `/ && section ~ /Further reading/ { first($0); next }
    /^\|/ {
      if ($0 ~ /^[|: -]+$/) next
      n = split($0, cells, "|")
      if (!header) { header = 1; split("", head); for (i = 2; i < n; i++) head[i] = trim(cells[i]); next }
      for (i = 2; i < n; i++) {
        if (head[i] == "Path") first(cells[i])
        else if (head[i] == "Read first") emit("S", cells[i])
        else emit("P", cells[i])
      }
      next
    }
    { emit("P", $0) }' "$1"
}

looks_like_path() {  # looks_like_path <folder> <word>: a prose word worth checking
  [ -n "$LEARNING_DIR" ] && case "$2" in "$LEARNING_DIR"|"$LEARNING_DIR"/*) return 1 ;; esac  # often local only
  case "$2" in
    *" "*|*://*|*"<"*|*">"*|*'$'*|*"{"*|*"}"*|*"@"*|*"="*|*"("*|*")"*|*"'"*|*'"'*|*","*|*";"*|-*|*\\*) return 1 ;;
    ./*|../*|*/) return 0 ;;
  esac
  case "$2" in
    */*) first=${2%%/*}
         { [ -e "$1/$first" ] || [ -e "$first" ]; } && return 0 ;;
  esac
  ext=${2##*.}
  [ "$ext" != "$2" ] || return 1
  case " $DOC_EXTENSIONS $CODE_EXTENSIONS " in *" $ext "*) return 0 ;; esac
  return 1
}
anywhere() {  # anywhere <path>: some file in the repo ends with this path
  [ -n "$(git ls-files -co --exclude-standard -- ":(glob)**/${1%/}" ":(glob)**/${1%/}/**" 2>/dev/null | head -1)" ]
}

if [ "$ANCHORS" = 1 ]; then
  for map in "$@"; do
    folder=$(dirname "$map")
    map_tokens "$map" | while IFS="$TAB" read -r kind line word; do
      path=${word%/}
      [ -n "$path" ] || continue
      [ "$kind" = S ] || looks_like_path "$folder" "$word" || continue
      for target in "$(rel "$folder" "$path")" "$path"; do
        if exists . "$target"; then printf '%s\t%s\t%s\n' "$map" "$line" "$target"; break; fi
      done
    done
  done
  exit 0
fi

check_map() {  # check_map <map> <max lines>
  map=$1
  folder=$(dirname "$map")
  MAP_COUNT=$((MAP_COUNT + 1))
  own=$(yes_if is_changed "$map")
  lines=$(wc -l < "$map" | tr -d ' ')
  [ "$lines" -le "$2" ] || report "$own" "$map has $lines lines (max $2)" \
    "move detail into $UNIT_DOC (or the doc it points to); keep the map"
  map_tokens "$map" > "$TMP_TOKENS"
  while IFS="$TAB" read -r kind line word; do
    path=${word%/}
    [ -n "$path" ] || continue
    if [ "$kind" = S ]; then
      exists "$folder" "$path" && continue
    else
      looks_like_path "$folder" "$word" || continue
      { exists "$folder" "$path" || exists . "$path" || anywhere "$path"; } && continue
    fi
    target=$(rel "$folder" "$path")
    caused=$own; was_removed "$target" && caused=yes
    report "$caused" "$map names a missing path: $path (line $line)" \
      "fix the path or remove it; to keep a word that is not a path, end the line with <!-- agent-kit: skip -->"
  done < "$TMP_TOKENS"
  if [ "$CHECK_ENV" = yes ]; then
    awk -F "$TAB" '$3 ~ /^[A-Z][A-Z0-9]*_[A-Z0-9_]+$/ { print $2 "\t" $3 }' "$TMP_TOKENS" | LC_ALL=C sort -u -k2,2 > "$TMP_ENV"
    while IFS="$TAB" read -r line name; do
      git grep --untracked -qw -e "$name" -- ':(exclude)*.md' 2>/dev/null ||
        report "$own" "$map names $name (line $line), which no code file uses" \
          "fix the name, or remove it if the setting is gone"
    done < "$TMP_ENV"
  fi
  [ "$MAPS" = beside ] && return  # no pointer files: wire.sh connects the maps locally
  for pointer in $POINTER_FILES; do
    if ! grep -q '@AGENTS.md' "$folder/$pointer" 2>/dev/null; then
      caused=$own; is_changed "$(rel "$folder" "$pointer")" && caused=yes
      report "$caused" "$folder/$pointer does not import the map" "create it containing exactly: @AGENTS.md"
    fi
  done
}

unit_sizes() {  # unit_sizes <units>: unit, code files and code lines, one line each
  units=$1
  [ -n "$units" ] || return 0
  # shellcheck disable=SC2086  # one pathspec per unit
  git -c core.quotepath=off grep --untracked -I -c '' -- $units 2>/dev/null |
    awk -v units="$units" -v exts="$CODE_EXTENSIONS" '
      BEGIN { n = split(units, u, " "); for (i = 1; i <= n; i++) { unit[u[i]] = 1; files[u[i]] = 0; lines[u[i]] = 0 }
              n = split(exts, e, " "); for (i = 1; i <= n; i++) ext[e[i]] = 1 }
      {
        match($0, /:[0-9]+$/); count = substr($0, RSTART + 1); path = substr($0, 1, RSTART - 1)
        k = split(path, parts, "/"); dot = split(parts[k], name, ".")
        if (dot < 2 || !(name[dot] in ext)) next
        prefix = ""; owner = ""
        for (i = 1; i < k; i++) { prefix = prefix (i > 1 ? "/" : "") parts[i]; if (prefix in unit) owner = prefix }
        if (owner != "") { files[owner]++; lines[owner] += count }
      }
      END { for (x in unit) print x "\t" files[x] "\t" lines[x] }'
}

listed() {  # listed <folder> <list>: the folder is in a config list
  for entry in $2; do [ "${entry%/}" = "$1" ] && return 0; done
  return 1
}
needs_map() {  # needs_map <unit>: big enough (or listed) to need its own map
  listed "$1" "$MAP_EXEMPT" && return 1
  listed "$1" "$MAP_REQUIRED" && return 0
  [ -n "$MAP_MIN_FILES$MAP_MIN_LINES" ] || return 0
  size=$(awk -F "$TAB" -v u="$1" '$1 == u { print $2 " " $3; exit }' "$TMP_SIZES")
  files=${size% *}; lines=${size#* }
  [ -n "$MAP_MIN_FILES" ] && [ "${files:-0}" -ge "$MAP_MIN_FILES" ] && return 0
  [ -n "$MAP_MIN_LINES" ] && [ "${lines:-0}" -ge "$MAP_MIN_LINES" ] && return 0
  return 1
}
reachable() {  # reachable <unit>: its parent folder has a map, or a map names it or its parent
  parent=$(dirname "$1")
  [ "$parent" != . ] && [ -f "$parent/$MAP_FILE" ] && return 0
  for target in "$1" "$parent"; do
    [ "$target" = . ] && continue
    while IFS= read -r map <&4; do
      dir=$(dirname "$map")
      if [ "$dir" = . ]; then name=$target
      else case "$target" in "$dir"/*) name=${target#"$dir"/} ;; *) continue ;; esac
      fi
      grep -qF "\`$name" "$map" && return 0
    done 4< "$TMP_MAPS"
  done
  return 1
}

relevant_maps() {  # the maps a scoped run must read: changed, or above a removed path
  printf '%s\n' "$CHANGED" > "$TMP/changed"
  printf '%s\n' "$GONE" > "$TMP/gone"
  awk -v pointers="$POINTER_FILES" -v map="$MAP_FILE" '
    FILENAME == ARGV[1] { if (NF) changed[$0]; next }
    FILENAME == ARGV[2] { if (NF) gone[$0]; next }
    {
      folder = substr($0, 1, length($0) - length(map)); sub(/\/$/, "", folder)
      if ($0 in changed) { print; next }
      n = split(pointers, p, " ")
      for (i = 1; i <= n; i++) if ((folder == "" ? p[i] : folder "/" p[i]) in changed) { print; next }
      for (path in gone) if (folder == "" || index(path, folder "/") == 1) { print; next }
    }' "$TMP/changed" "$TMP/gone" "$TMP_MAPS"
}

check_maps() {
  if [ ! -f "$MAP_FILE" ]; then
    fail "$MAP_FILE is missing at the repo root" "write the root map (BOOTSTRAP.md, phase 4)"
    return
  fi
  find_maps > "$TMP_MAPS"
  if [ -n "$SCOPE" ]; then  # a scoped run skips maps this change cannot break
    relevant_maps > "$TMP/relevant"; mv "$TMP/relevant" "$TMP_MAPS.scoped"
  else
    cp "$TMP_MAPS" "$TMP_MAPS.scoped"
  fi
  while IFS= read -r map <&3; do
    case "$map" in
      */*/*) max=$UNIT_MAP_MAX ;;  # deeper than a first-level folder
      *) max=$TOP_MAP_MAX ;;
    esac
    check_map "$map" "$max"
  done 3< "$TMP_MAPS.scoped"
  units=$(unit_folders)
  if [ -n "$SCOPE" ]; then  # a scoped run looks only at units this change touched
    printf '%s\n' "$CHANGED" > "$TMP/changed"
    units=$(printf '%s\n' "$units" | awk 'NR == FNR { if (NF) changed[$0]; next }
      { for (path in changed) if (index(path, $0 "/") == 1) { print; next } }' "$TMP/changed" -)
  fi
  unit_sizes "$(echo $units)" > "$TMP_SIZES"
  for unit in $units; do
    inside=$(yes_if changed_under "$unit")
    if needs_map "$unit"; then
      for needed in "$UNIT_DOC" "$MAP_FILE"; do
        [ -f "$unit/$needed" ] || report "$inside" "$unit has no $needed" \
          "create it with the scaffold skill (or add $unit to MAP_EXEMPT in $CONF)"
      done
    elif [ ! -f "$unit/$MAP_FILE" ] && ! reachable "$unit"; then
      report "$inside" "no map names $unit, so agents cannot find it" \
        "add a row for \`$unit/\` (or its parent folder) to the nearest $MAP_FILE"
    fi
  done
}

# --- Skills, sharing, git, portability ---------------------------------------------
tree_sum() {  # tree_sum <folder>: one checksum line per file, for comparing copies
  (cd "$1" && find . -type f | LC_ALL=C sort | while IFS= read -r file; do
    printf '%s %s\n' "$file" "$(cksum < "$file")"; done)
}

check_skills() {
  [ -d .agents/skills ] || return
  if [ "$MAPS" = beside ] && [ -d .claude/skills ] && [ ! -L .claude/skills ]; then
    for skill in .agents/skills/*/; do  # kit skills copied beside the team's skills
      name=${skill%/}; name=${name##*/}
      copy=".claude/skills/$name"
      [ -f "$copy/.agent-kit" ] || continue
      [ "$(tree_sum "$copy" | grep -v '^\./\.agent-kit ')" = "$(tree_sum "${skill%/}")" ] ||
        fail "$copy is out of sync with .agents/skills/$name" "run: sh .agents/kit/sync-skills.sh"
    done
    return
  fi
  if [ -f .claude/skills ]; then
    # A symlink checked out where links are off (Windows) becomes a text file.
    fail ".claude/skills is a plain file: this clone cannot use symlinks" \
      "set SKILLS_LINK=\"copy\" in .agents/agent-kit.conf, run: rm .claude/skills && sh .agents/kit/sync-skills.sh, and commit"
  elif [ ! -e .claude/skills ]; then
    [ -d .claude ] && fail ".claude/skills is missing, so Claude Code sees no skills" "run: sh .agents/kit/sync-skills.sh"
  elif [ "$SKILLS_LINK" = "copy" ]; then
    [ "$(tree_sum .claude/skills)" = "$(tree_sum .agents/skills)" ] ||
      fail ".claude/skills is out of sync with .agents/skills" \
        "edit skills in .agents/skills only, then run: sh .agents/kit/sync-skills.sh"
  else
    claude=$(cd -P .claude/skills && pwd)
    agents=$(cd -P .agents/skills && pwd)
    [ "$claude" = "$agents" ] || fail ".claude/skills is a copy, not a link to .agents/skills; copies drift" \
      "run: sh .agents/kit/sync-skills.sh (or set SKILLS_LINK=\"copy\" where symlinks do not work)"
  fi
}

kit_local_paths() {  # beside mode: pathspecs of everything the kit makes on this computer
  printf '%s\n' ":(glob)**/*${DOC_SUFFIX:-.agentk.md}" .agents .claude/settings.local.json .cursor/rules/agent-kit.mdc \
    ':(glob)**/CLAUDE.local.md' ':(glob)**/AGENTS.override.md'
  [ -L .claude/skills ] && printf '%s\n' .claude/skills
  for skill in .agents/skills/*/; do
    [ -d "$skill" ] || continue
    name=${skill%/}; name=${name##*/}
    [ -f ".claude/skills/$name/.agent-kit" ] && printf '%s\n' ".claude/skills/$name"
  done
  return 0
}
kit_made() {  # kit_made <tracked path>: the kit wrote it (not a team file of the same name)
  case "$1" in
    *CLAUDE.local.md|*AGENTS.override.md|*agent-kit.mdc) git show ":$1" 2>/dev/null | grep -q 'agent-kit' ;;
    *) return 0 ;;
  esac
}

agent_paths() {  # pathspecs of every agent file: maps, pointers, lineage, kit folders
  if [ "$MAPS" = beside ]; then kit_local_paths; return; fi
  printf '%s\n' AGENTS.md ':(glob)**/AGENTS.md' "$DECISIONS_INDEX" "$PLAN_DOC" .agents .claude .codex .githooks
  for pointer in $POINTER_FILES; do printf '%s\n' "$pointer" ":(glob)**/$pointer"; done
  [ -z "$LEDGER" ] || printf '%s\n' "$LEDGER"
  [ -z "$DECISIONS_DIR" ] || printf '%s\n' "$DECISIONS_DIR"
  if [ -n "$DOC_SUFFIX" ]; then  # every kit doc, wherever it is
    printf '%s\n' ":(glob)**/*$DOC_SUFFIX"
    [ "$SHARE" = learning ] && [ -n "$LEARNING_DIR" ] && printf '%s\n' ":(exclude)$LEARNING_DIR"
  fi
}

check_share_mode() {  # files reach the remote only when SHARE allows it
  case "$SHARE" in
    agent|all|learning|none) ;;
    *) fail "SHARE=\"$SHARE\" is not a share mode" "set SHARE to agent, all, learning or none in $CONF"; return ;;
  esac
  if [ "$MAPS" = beside ]; then
    if [ "$SHARE" != none ]; then
      fail "MAPS=beside keeps your setup on this computer, but SHARE=$SHARE would push it" \
        "set SHARE=\"none\" in $CONF, then run: sh <agent_dev_kit>/install.sh --maps beside ."
      return
    fi
    set -f
    # shellcheck disable=SC2046
    leaked=$(git ls-files -- $(kit_local_paths) | while IFS= read -r path; do kit_made "$path" && printf '%s ' "$path"; done)
    set +f
    [ -z "$leaked" ] || fail "your local setup is tracked or staged, so it would reach the team's repo: $leaked" \
      "run: git rm -r --cached -q -- $leaked (the files stay on disk)"
    { git check-ignore -q "$MAP_FILE" && git check-ignore -q .agents/kit/check.sh; } || fail "your local setup is not ignored, so it can be committed by mistake" \
      "run once: sh <agent_dev_kit>/install.sh --maps beside .  (it writes .git/info/exclude)"
    return
  fi
  if [ "$SHARE" = agent ] || [ "$SHARE" = all ]; then
    untracked=$(git ls-files --others --exclude-standard -- .agents .claude .codex .githooks | tr '\n' ' ')
    [ -z "$untracked" ] || fail "agent files are not in git, so a fresh clone lacks them: $untracked" \
      "git add them (put personal files in .gitignore or your global git ignore)"
  else
    set -f  # the pathspecs contain ** for git, not for the shell
    # shellcheck disable=SC2046  # one pathspec per line
    tracked=$(git ls-files -- $(agent_paths) | head -5 | tr '\n' ' ')
    set +f
    [ -z "$tracked" ] || fail "SHARE=$SHARE keeps agent files local, but git tracks or stages: $tracked" \
      "run: git rm -r --cached -q --ignore-unmatch -- $(agent_paths | tr '\n' ' ') (the files stay on disk)"
    { git check-ignore -q AGENTS.md && git check-ignore -q .agents/kit/check.sh; } || fail "agent files are not ignored, so they can be committed by mistake" \
      "run once: sh <agent_dev_kit>/install.sh  (it writes .git/info/exclude for SHARE=$SHARE)"
  fi
}

hook_runs_check() {  # some hook manager runs check.sh before a commit
  hooks_dir=$(git config core.hooksPath || true)
  hooks_dir=${hooks_dir:-$(git rev-parse --git-path hooks)}
  for file in "$hooks_dir/pre-commit" .pre-commit-config.yaml lefthook.yml lefthook.yaml .lefthook.yml .husky/pre-commit; do
    grep -qs 'agents/kit/check.sh' "$file" && return 0
  done
  return 1
}

check_git_setup() {
  if [ "$CI" = 0 ] && [ "$MAPS" = beside ] && ! hook_runs_check; then
    [ "$QUIET" = 1 ] || echo "NOTE no pre-commit check runs here (the team's hook setup is not touched in beside mode); run sh .agents/kit/check.sh before you push"
  elif [ "$CI" = 0 ] && ! hook_runs_check; then
    fail "the pre-commit check is not active in this clone" \
      "run once: git config core.hooksPath .githooks  (or add .agents/kit/check.sh --staged to your hook manager; install.sh prints the line)"
  fi
  for config in .claude/settings.json .claude/settings.local.json .codex/hooks.json; do
    [ -f "$config" ] || continue
    if grep -qF "$ROOT" "$config" ||
       grep -qE '(/Users/|/home/|[A-Za-z]:[\\/]+Users)' "$config"; then
      fail "$config hard-codes a path on this machine" \
        "use \"\$CLAUDE_PROJECT_DIR/...\" or \"\$(git rev-parse --show-toplevel)/...\""
    fi
    for script in $(grep -oE '\.(agents|claude|codex|githooks)/[A-Za-z0-9_./-]+\.(sh|py)' "$config"); do
      [ -f "$script" ] || fail "$config runs $script, which does not exist" "fix the path in $config"
    done
  done
}

check_portability() {  # works on Windows, macOS and Linux clones alike
  eol=$(git check-attr eol -- .agents/kit/check.sh | sed 's/.*: //')
  [ "$eol" = "lf" ] || fail "scripts may get Windows line endings (CRLF), which breaks sh" \
    "add the agent-kit lines from agent_dev_kit/templates/config/gitattributes to .gitattributes"
  for script in .githooks/* .agents/kit/*.sh; do
    mode=$(git ls-files -s -- "$script" | cut -c1-6)
    [ -z "$mode" ] || [ "$mode" = "100755" ] || fail "$script is not executable in git; macOS and Linux clones skip it" \
      "run: git add --chmod=+x $script"
  done
}

check_learning_docs() {
  [ -n "$LEARNING_DIR" ] || return
  if [ "$SHARE" = all ] || [ "$SHARE" = learning ]; then
    if git check-ignore -q "$LEARNING_DIR/probe.md"; then
      fail "SHARE=$SHARE pushes learning docs, but $LEARNING_DIR/ is git-ignored" \
        "remove the $LEARNING_DIR/ rule from .gitignore or .git/info/exclude (install.sh rewrites its own)"
    fi
    return
  fi
  tracked=$(git ls-files -- "$LEARNING_DIR" | head -3 | tr '\n' ' ')
  [ -z "$tracked" ] || fail "learning docs are tracked or staged: $tracked" \
    "run: git rm -r --cached $LEARNING_DIR (they stay local)"
  git check-ignore -q "$LEARNING_DIR/probe.md" || fail "$LEARNING_DIR/ is not git-ignored" \
    "run: sh <agent_dev_kit>/install.sh  (it writes the ignore rule for SHARE=$SHARE)"
}

check_ledger() {  # every folder beside the ledger has a row with an outcome
  [ -n "$LEDGER" ] && [ -f "$LEDGER" ] || return
  rows=$(awk '
    { sub(/\r$/, "") }
    function trim(s) { gsub(/^[ \t\r]+|[ \t\r]+$/, "", s); return s }
    /^\|/ && $0 !~ /^[|: -]+$/ {
      n = split($0, cells, "|")
      if (!col) { for (i = 2; i < n; i++) if (trim(cells[i]) == "Outcome") col = i; next }
      if (match(cells[2], /`[^`]+`/)) {
        name = substr(cells[2], RSTART + 1, RLENGTH - 2); sub(/\/.*/, "", name)
        print name "\t" (trim(cells[col]) == "" ? "EMPTY" : "ok")
      }
    }' "$LEDGER")
  ledger_changed=$(yes_if is_changed "$LEDGER")
  for folder in "$(dirname "$LEDGER")"/*/; do
    [ -d "$folder" ] || continue
    name=$(basename "$folder")
    is_skipped "$name" && continue
    caused=$ledger_changed; changed_under "${folder%/}" && caused=yes
    row=$(printf '%s\n' "$rows" | awk -F '\t' -v n="$name" '$1 == n { print $2; exit }')
    if [ -z "$row" ]; then
      report "$caused" "$name has no row in $LEDGER" "add its row through the lineage skill"
    elif [ "$row" = "EMPTY" ]; then
      report "$caused" "$name has no outcome in $LEDGER" "summarise its report, or write: pending: not run yet"
    fi
  done
}

check_decisions() {  # one file per decision, and the index lists each with its status
  [ -n "$DECISIONS_DIR" ] || return
  caused=$(yes_if is_changed "$DECISIONS_INDEX"); changed_under "${DECISIONS_DIR%/}" && caused=yes
  [ -f "$DECISIONS_INDEX" ] || ls "${DECISIONS_DIR%/}"/D-*.md >/dev/null 2>&1 || return
  if [ ! -f "$DECISIONS_INDEX" ]; then
    report "$caused" "$DECISIONS_INDEX (the index of $DECISIONS_DIR/) is missing" \
      "create it from agent_dev_kit/templates/docs/DECISIONS.md"
    return
  fi
  index=$(awk '
    { sub(/\r$/, "") }
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    /^\|/ && $0 !~ /^[|: -]+$/ {
      n = split($0, cells, "|")
      if (!col) { for (i = 2; i < n; i++) if (trim(cells[i]) == "Status") col = i; next }
      if (match(cells[2], /D-[0-9]+/)) print substr(cells[2], RSTART, RLENGTH) "\t" trim(cells[col])
    }' "$DECISIONS_INDEX")
  ids=""
  for file in "${DECISIONS_DIR%/}"/D-*.md; do
    [ -f "$file" ] || continue
    id=$(basename "$file" | sed 's/^\(D-[0-9]*\).*/\1/')
    ids="$ids $id"
    status=$(sed -n 's/^status:[[:blank:]]*//p' "$file" | head -1 | tr -d '\r')
    row=$(printf '%s\n' "$index" | awk -F "$TAB" -v i="$id" '$1 == i { print $2; found = 1; exit } END { if (!found) print "<no row>" }')
    if [ "$row" = "<no row>" ]; then
      report "$caused" "$id ($file) has no row in $DECISIONS_INDEX" "add: | [$id]($file) | <decision> | $status | <date> |"
    elif [ "$row" != "$status" ]; then
      report "$caused" "$id is \"$status\" in $file but \"$row\" in $DECISIONS_INDEX" "make the index row match the file"
    fi
    case "$status" in
      "superseded by "*) target=${status#superseded by }
        ls "${DECISIONS_DIR%/}/$target"-*.md >/dev/null 2>&1 ||
          report "$caused" "$id is superseded by $target, which has no file" "write $target with the lineage skill, or fix the status" ;;
    esac
  done
  for id in $(printf '%s\n' "$index" | cut -f1); do
    case " $ids " in *" $id "*) ;; *)
      report "$caused" "$DECISIONS_INDEX lists $id, but $DECISIONS_DIR/ has no $id-*.md" "write the file or remove the row" ;;
    esac
  done
}

check_fixed_names() {  # maps, pointers and skills keep their exact names
  [ -n "$DOC_SUFFIX" ] || return
  fixed="SKILL"; [ "$MAPS" = beside ] || fixed="AGENTS SKILL"
  for pointer in $POINTER_FILES; do fixed="$fixed ${pointer%.md}"; done
  if [ -n "$SCOPE" ]; then
    printf '%s\n' "$CHANGED" > "$TMP/candidates"
  else
    set -- -name '.?*' -o -name '_*'
    for skip in $SKIP_DIRS; do set -- "$@" -o -name "$skip"; done
    find . -type d \( "$@" \) -prune -o -type f -name "*$DOC_SUFFIX" -print | sed 's|^\./||' > "$TMP/candidates"
    find .agents/skills -type f -name "*$DOC_SUFFIX" 2>/dev/null >> "$TMP/candidates"
  fi
  while IFS= read -r file; do
    base=${file##*/}
    for name in $fixed; do
      [ "$base" = "$name$DOC_SUFFIX" ] || continue
      report "$(yes_if is_changed "$file")" "$file must keep its exact name: agents load only $name.md" \
        "rename it: git mv $file $(dirname "$file")/$name.md"
    done
  done < "$TMP/candidates"
}

run_commands() {  # the ## Run and ## Check blocks of RUN_COMMANDS_MAPS succeed
  [ "$RUN_COMMANDS" = 1 ] || return
  for map in $RUN_COMMANDS_MAPS; do
    if [ ! -f "$map" ]; then
      fail "RUN_COMMANDS_MAPS names $map, which does not exist" "fix RUN_COMMANDS_MAPS in $CONF"; continue
    fi
    awk '{ sub(/\r$/, "") }
         /^#/ { section = ($0 ~ /^## (Run|Check)[ \t]*$/); next }
         section && /^[ \t]*(```|~~~)/ { code = !code; next }
         section && code { print }' "$map" > "$TMP_CMDS"
    [ -s "$TMP_CMDS" ] || continue
    if ! out=$(cd "$(dirname "$map")" && sh -e "$TMP_CMDS" 2>&1); then
      fail "the commands under ## Run / ## Check in $map fail: $(printf '%s' "$out" | tail -3 | tr '\n' ' ')" \
        "fix the commands in $map, or the code they run"
    fi
  done
}

check_wiring() {  # beside mode: the local files that connect your maps to each agent are fresh
  [ "$MAPS" = beside ] && [ -f .agents/kit/wire.sh ] || return
  out=$(sh .agents/kit/wire.sh --check 2>&1) && return
  fail "the local wiring of your maps is stale or missing: $(printf '%s' "$out" | head -3 | tr '\n' ' ')" \
    "run: sh .agents/kit/wire.sh"
}

notes() {  # hints that never fail: a newer kit, code written without its docs
  [ "$QUIET" = 1 ] && return
  kit=${AGENT_KIT_HOME:-$HOME/agent_dev_kit}
  if [ -f "$kit/VERSION" ] && [ -f .agents/kit/VERSION ]; then
    have=$(cat .agents/kit/VERSION); new=$(cat "$kit/VERSION")
    if awk -v a="$have" -v b="$new" 'BEGIN { n = split(a, x, "."); m = split(b, y, ".")
         for (i = 1; i <= (n > m ? n : m); i++) { if (x[i] + 0 < y[i] + 0) exit 0; if (x[i] + 0 > y[i] + 0) exit 1 }
         exit 1 }'; then
      echo "NOTE this repo has kit $have; $new is available: run agent-kit upgrade"
    fi
  fi
  if [ "$MAPS" = beside ]; then
    case " $AGENTS_USED " in *" gemini "*|*" antigravity "*)
      if git ls-files --error-unmatch .gemini/settings.json >/dev/null 2>&1 && grep -q '"fileName"' .gemini/settings.json; then
        echo "NOTE the team's .gemini/settings.json sets context.fileName, so Gemini does not load $MAP_FILE here (see the ~/.gemini/GEMINI.md fallback)"
      elif ! grep -qs "$MAP_FILE" "$HOME/.gemini/settings.json"; then
        echo "NOTE Gemini does not load $MAP_FILE yet: add it to context.fileName in ~/.gemini/settings.json (agent-kit wire offers it)"
      fi ;;
    esac
  fi
  pending="$(git rev-parse --git-common-dir)/agent-kit/pending"
  if [ -s "$pending" ]; then
    echo "NOTE $(wc -l < "$pending" | tr -d ' ') commit(s) changed code without its docs: run the catch-up skill"
  fi
}

TMP=$(mktemp -d) || exit 1
trap 'rm -rf "$TMP"' EXIT
TMP_MAPS="$TMP/maps" TMP_TOKENS="$TMP/tokens" TMP_ENV="$TMP/env" TMP_SIZES="$TMP/sizes" TMP_CMDS="$TMP/cmds"
check_maps
check_skills
check_share_mode
check_git_setup
check_portability
check_learning_docs
check_ledger
check_decisions
check_fixed_names
check_wiring
run_commands
notes

if [ "$FAILS" -gt 0 ]; then
  echo "agent-kit: $FAILS check(s) failed"
  [ "$WARNS" -eq 0 ] || echo "agent-kit: $WARNS more problem(s) outside this change (see: sh .agents/kit/check.sh)"
  exit 1
fi
if [ "$WARNS" -gt 0 ]; then
  echo "agent-kit: $WARNS problem(s) outside this change do not block (see: sh .agents/kit/check.sh)"
elif [ "$QUIET" = 0 ]; then
  echo "agent-kit: all checks pass ($MAP_COUNT maps)"
fi
