#!/bin/sh
# Suggest map rows for a folder: its subfolders and entry files that its
# AGENTS.md does not name yet. The structure comes from the tree; the agent
# only writes the "What" column. Read only.
#
# Usage: sh .agents/kit/map-rows.sh [folder]
cd "$(git rev-parse --show-toplevel)" || exit 1
SKIP_DIRS="node_modules .venv venv vendor target build dist out bin obj"
LEARNING_DIR="learning_docs"
DOC_SUFFIX=""
[ -f .agents/agent-kit.conf ] && . ./.agents/agent-kit.conf
if [ "${MAPS:-own}" = beside ]; then MAP_FILE="AGENTS${DOC_SUFFIX:-.agentk.md}"; else MAP_FILE="AGENTS.md"; fi
folder=${1:-.}
folder=${folder%/}
named=""
[ -f "$folder/$MAP_FILE" ] && named=$(sh .agents/kit/check.sh --anchors "$folder/$MAP_FILE" | cut -f3)

is_named() { printf '%s\n' "$named" | grep -qxF -- "$1"; }
rel() { if [ "$folder" = . ]; then printf '%s' "$1"; else printf '%s/%s' "$folder" "$1"; fi; }

echo "| Path | What |"
echo "|---|---|"
for child in "$folder"/*/; do
  [ -d "$child" ] || continue
  name=$(basename "$child")
  case "$name" in .*|_*) continue ;; esac
  [ "$name" = "$LEARNING_DIR" ] && continue
  skip=no; for dir in $SKIP_DIRS; do [ "$name" = "$dir" ] && skip=yes; done
  [ "$skip" = yes ] && continue
  is_named "$(rel "$name")" || printf '| `%s/` | <what it holds> |\n' "$name"
done
for file in "$folder"/*; do
  [ -f "$file" ] || continue
  name=$(basename "$file")
  case "$name" in
    package.json|pyproject.toml|setup.py|go.mod|Cargo.toml|pom.xml|build.gradle*|CMakeLists.txt|\
    Makefile|justfile|Dockerfile|docker-compose*.y*ml|compose*.y*ml|*.csproj|*.sln|\
    main.*|index.*|app.*|server.*|cli.*|__main__.py|manage.py)
      is_named "$(rel "$name")" || printf '| `%s` | <what it does> |\n' "$name" ;;
  esac
done
