#!/bin/sh
# Interactive learning docs: which pages are missing, stale or orphaned.
# Read only. The interactive-docs skill starts from it.
#
# Usage (from anywhere in the repo):
#   sh .agents/skills/interactive-docs/status.sh
#
# Prints one line per page: <state> TAB <page> TAB <learning doc>
#   missing   the learning doc has no page yet
#   stale     the learning doc was refreshed after the page was built
#             (its source_commit differs from the one in the page's marker)
#   current   the page matches its learning doc
#   orphan    the page's learning doc is gone
# and adds "(learning doc stale)" when the learning doc's own code changed:
# refresh it with learning-docs (or docs-sync) first, then the page.
#
# Each page starts with a marker that records what it was built from:
#   <!-- agent-kit: interactive from=01-api.agentk.md source_commit=abc1234 -->
# Pages live in <LEARNING_DIR>/interactive_l_docs/, one per learning doc, at
# the same relative path with .html in place of the doc suffix.
# The folders interactive_l_docs/ and architecture/ are not learning docs.
cd "$(git rev-parse --show-toplevel)" || exit 1
LEARNING_DIR="learning_docs"
DOC_SUFFIX=""
[ -f .agents/kit/lib.sh ] && . ./.agents/kit/lib.sh && load_conf
TAB=$(printf '\t')
PAGES="$LEARNING_DIR/interactive_l_docs"

if [ -z "$LEARNING_DIR" ] || [ ! -d "$LEARNING_DIR" ]; then
  echo "interactive-docs: no learning docs in ${LEARNING_DIR:-<LEARNING_DIR is empty>}/. Run the learning-docs skill first."
  exit 0
fi

front() {  # front <doc> <key>: one value from the YAML front matter
  tr -d '\r' < "$1" | awk -v key="$2" '
    NR == 1 && /^---[[:space:]]*$/ { on = 1; next }
    on && /^---[[:space:]]*$/ { exit }
    on && index($0, key ":") == 1 { sub(/^[^:]*:[[:space:]]*/, ""); gsub(/["\047]/, ""); print; exit }'
}
marker() {  # marker <page> <field>: one field of the page's agent-kit marker
  tr -d '\r' < "$1" | sed -n "s/.*<!-- agent-kit: interactive .*$2=\([^ ]*\).*-->.*/\1/p" | head -1
}
page_of() {  # page_of <learning doc>: its page path
  rel=${1#"$LEARNING_DIR"/}
  case "$rel" in
    *"${DOC_SUFFIX:-.agentk.md}") rel=${rel%"${DOC_SUFFIX:-.agentk.md}"} ;;
    *.md) rel=${rel%.md} ;;
  esac
  echo "$PAGES/$rel.html"
}

stale_docs=""
[ -f .agents/kit/stale.sh ] && stale_docs=$(sh .agents/kit/stale.sh 2>/dev/null | cut -f1)

docs=$(find "$LEARNING_DIR" \( -path "$PAGES" -o -path "$LEARNING_DIR/architecture" \) -prune -o -name '*.md' -type f -print | LC_ALL=C sort)
for doc in $docs; do
  commit=$(front "$doc" source_commit)
  [ -n "$commit" ] || continue  # not a learning doc (no front matter)
  page=$(page_of "$doc")
  if [ ! -f "$page" ]; then state=missing
  elif [ "$(marker "$page" source_commit)" = "$commit" ]; then state=current
  else state=stale
  fi
  note=""
  case "
$stale_docs
" in *"
$doc
"*) note=" (learning doc stale)" ;; esac
  printf '%s%s%s%s%s%s\n' "$state" "$TAB" "$page" "$TAB" "$doc" "$note"
done

if [ -d "$PAGES" ]; then
  for page in $(find "$PAGES" -name '*.html' -type f | LC_ALL=C sort); do
    from=$(marker "$page" from)
    [ -n "$from" ] || continue  # index.html and pages without a marker
    [ -f "$LEARNING_DIR/$from" ] || printf 'orphan%s%s%s%s\n' "$TAB" "$page" "$TAB" "$LEARNING_DIR/$from"
  done
fi
exit 0
