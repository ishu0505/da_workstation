#!/bin/sh
# Make .claude/skills show the skills kept in .agents/skills.
# SKILLS_LINK="symlink" (default): a link, so there is only one copy.
# SKILLS_LINK="copy": a real copy, for clones where symlinks do not work
# (Windows without Developer Mode). Run again after editing a skill.
set -eu
cd "$(git rev-parse --show-toplevel)"
SKILLS_LINK="symlink"
MAPS="own"
[ -f .agents/agent-kit.conf ] && . ./.agents/agent-kit.conf
per_skill() {  # beside mode: each kit skill in its own marked folder, beside any team skill
  mkdir -p .claude/skills
  for skill in .agents/skills/*/; do
    [ -d "$skill" ] || continue
    name=${skill%/}; name=${name##*/}
    dest=".claude/skills/$name"
    if [ -e "$dest" ] && [ ! -f "$dest/.agent-kit" ]; then
      echo "skipped $dest: the team has a skill with this name"
      continue
    fi
    rm -rf "$dest"
    cp -R "${skill%/}" "$dest"
    : > "$dest/.agent-kit"
    echo "copied $skill -> $dest"
  done
}
if [ "$MAPS" = beside ] && { { [ -d .claude/skills ] && [ ! -L .claude/skills ]; } || [ "$SKILLS_LINK" = copy ]; }; then
  per_skill  # a team repo: never replace a .claude/skills folder the team may own
  exit 0
fi
mkdir -p .claude
rm -rf .claude/skills
if [ "$SKILLS_LINK" = "symlink" ]; then
  ln -s ../.agents/skills .claude/skills 2>/dev/null || true
  if [ -L .claude/skills ]; then echo "linked .claude/skills -> ../.agents/skills"; exit 0; fi
  # Git Bash on Windows copies instead of linking: switch this repo to copy mode.
  rm -rf .claude/skills
  sed 's/^SKILLS_LINK=.*/SKILLS_LINK="copy"/' .agents/agent-kit.conf > .agents/agent-kit.conf.tmp &&
    mv .agents/agent-kit.conf.tmp .agents/agent-kit.conf
  grep -q '^SKILLS_LINK=' .agents/agent-kit.conf || printf '\nSKILLS_LINK="copy"\n' >> .agents/agent-kit.conf
  echo "symlinks do not work here: set SKILLS_LINK=\"copy\" in .agents/agent-kit.conf"
  if [ "$MAPS" = beside ]; then per_skill; exit 0; fi
fi
cp -R .agents/skills .claude/skills
echo "copied .agents/skills -> .claude/skills"
