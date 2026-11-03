#!/bin/sh
# Copy the demo fixture to a fresh folder so each rehearsal starts from the planted bug.
# Usage: sh demo/prepare.sh [target]   (default: $HOME/da-demo-pricing)
set -eu
here=$(cd "$(dirname "$0")" && pwd)
target=${1:-$HOME/da-demo-pricing}
case "$target" in
  "$HOME"/da-demo-*|/tmp/da-demo-*) ;;
  *) echo "Refusing: target must be \$HOME/da-demo-* or /tmp/da-demo-*" >&2; exit 2 ;;
esac
rm -rf "$target"
mkdir -p "$target"
cp "$here/fixture/pricing.py" "$here/fixture/test_pricing.py" "$target/"
echo "Demo project ready: $target"
echo "Sidebar task: @$target run the tests, find the bug, fix it, and run the tests again"
