"""Stop hook: keep the docs true without slowing the agent down.

Run by `.agents/kit/stop-hook.sh` when an agent ends a turn, and with
`--baseline` by `session-start.sh` when a session starts. It finds what the
turn changed from git (a snapshot of `git status` and HEAD per session), so
shell edits, subagents and every agent are covered, and:

1. when nothing changed, does nothing: no check, no message;
2. runs `.agents/kit/check.sh --changed <files>` and blocks the stop only on
   problems those files cause;
3. asks for the docs step once per file, and only when a changed code file
   is named in a map, is an interface file (INTERFACE_GLOBS), or was added or
   removed in a folder a map names; an edit to an AGENTS.md, the kit's
   detail doc (README.agentk.md) or a README.md above the file satisfies it;
4. records learning docs whose `source_paths` changed in
   `.git/agent-kit/stale-learning` for the wrap-up skill; never blocks for it;
5. with MAPS="beside" (a team repo): refreshes the local wiring when a map
   changed (`.agents/kit/wire.sh`), and asks once when the turn edited one
   of the team's own agent files (AGENTS.md, CLAUDE.md, GEMINI.md).

`stop_hook_active` is set on the follow-up stop, so it never loops. Any
error lets the stop through. Installed by agent_dev_kit; do not edit here.
"""

from __future__ import annotations

import fnmatch
import json
import posixpath
import re
import subprocess
import sys
import time
from pathlib import Path

CHANGE_TOOLS = {"Edit", "Write", "MultiEdit", "NotebookEdit", "Bash"}
SKIP_DIRS = "node_modules .venv venv vendor target build dist out bin obj"
CHECKS_FAILING = (
    "The agent-kit checks fail for files this turn changed (.agents/kit/check.sh). "
    "Fix what they name before ending the turn:\n{failures}"
)
TEAM_FILE = (
    "These files belong to the team (this repo works beside them): {files}. Put your change in "
    "{map_file} instead, unless the user asked you to edit the team's file. If they did, say so in one line."
)
TEAM_NAMES = ("AGENTS.md", "CLAUDE.md", "GEMINI.md")
REMINDER = (
    "Docs step: this turn changed code that the docs describe:\n{items}\n"
    "Update the map or {unit_doc} named above in the same change (never a Markdown file the "
    "kit did not write), and run the lineage skill for a decision. If no doc needs to change, "
    "say so in one line."
)


def git(repo: Path, *args: str) -> str:
    run = subprocess.run(["git", "-c", "core.quotepath=off", *args], cwd=repo,
                         capture_output=True, text=True, timeout=30)
    return run.stdout if run.returncode == 0 else ""


def read_config(repo: Path) -> dict[str, str]:
    """KEY="value" lines of .agents/agent-kit.conf."""
    config = {}
    path = repo / ".agents" / "agent-kit.conf"
    for line in path.read_text().splitlines() if path.exists() else []:
        match = re.match(r'^([A-Z_]+)="?([^"]*)"?\s*$', line.strip())
        if match:
            config[match.group(1)] = match.group(2)
    return config


def snapshot(repo: Path) -> dict:
    """HEAD, and the status, size and mtime of every file git status lists."""
    out = subprocess.run(["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
                         cwd=repo, capture_output=True, timeout=30).stdout.decode("utf-8", "replace")
    entries = out.split("\0")
    files: dict[str, str] = {}
    i = 0
    while i < len(entries):
        entry = entries[i]
        i += 1
        if len(entry) < 4:
            continue
        status, path = entry[:2], entry[3:]
        if status[0] in "RC":
            i += 1  # the next entry is the old name
        try:
            stat = (repo / path).stat()
            files[path] = f"{status}:{stat.st_size}:{stat.st_mtime_ns}"
        except OSError:
            files[path] = f"{status}:gone"
    return {"head": git(repo, "rev-parse", "-q", "--verify", "HEAD").strip(), "files": files}


def changes_since(repo: Path, before: dict, now: dict) -> dict[str, str]:
    """Path -> A (added), D (removed) or M (changed), between two snapshots."""
    found: dict[str, str] = {}
    if before["head"] and now["head"] and before["head"] != now["head"]:
        for line in git(repo, "diff", "--name-status", before["head"], now["head"]).splitlines():
            parts = line.split("\t")
            if len(parts) >= 2:
                found[parts[-1]] = parts[0][0] if parts[0][0] in "AD" else "M"
                if parts[0][0] == "R":
                    found[parts[1]] = "D"
                    found[parts[-1]] = "A"
    for path, sign in now["files"].items():
        if before["files"].get(path) != sign:
            status = sign.split(":")[0]
            found[path] = "D" if sign.endswith(":gone") else "A" if ("?" in status or "A" in status) else "M"
    for path in before["files"]:
        if path not in now["files"]:
            found.setdefault(path, "D" if not (repo / path).exists() else "M")
    return found


def is_user_prompt(entry: dict) -> bool:
    content = (entry.get("message") or {}).get("content")
    if entry.get("type") != "user":
        return False
    if isinstance(content, str):
        return True
    return isinstance(content, list) and not any(
        isinstance(block, dict) and block.get("type") == "tool_result" for block in content)


def turn_used_change_tools(transcript: str | None) -> bool:
    """Only for the first stop of a session with no baseline: did this turn edit?
    An unreadable transcript (another agent's format) counts as yes."""
    if not transcript or not Path(transcript).exists():
        return True
    calls: list[dict] = []
    for line in Path(transcript).read_text().splitlines():
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if is_user_prompt(entry):
            calls = []
        content = (entry.get("message") or {}).get("content")
        if entry.get("type") == "assistant" and isinstance(content, list):
            calls += [block for block in content
                      if isinstance(block, dict) and block.get("type") == "tool_use"]
    return any(call.get("name") in CHANGE_TOOLS for call in calls)


def failing_checks(repo: Path, paths: list[str]) -> str:
    """Output of check.sh for these paths when it fails, else an empty string."""
    try:
        run = subprocess.run(["sh", ".agents/kit/check.sh", "--quiet", "--changed", *paths],
                             cwd=repo, capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired):
        return ""  # a missing shell or a hung run must not trap the agent
    return run.stdout[:3000] if run.returncode else ""


def skipped(path: str, config: dict[str, str]) -> bool:
    names = set(config.get("SKIP_DIRS", SKIP_DIRS).split()) | {config.get("LEARNING_DIR", "learning_docs")}
    return any(part in names or part[:1] in (".", "_") for part in path.split("/")[:-1])


def unit_doc(config: dict[str, str]) -> str:
    """The kit's detail doc in each unit: README.agentk.md, or README.md before kit 0.6."""
    return "README" + (config.get("DOC_SUFFIX") or ".md")


def map_file(config: dict[str, str]) -> str:
    """The kit's map: AGENTS.md, or AGENTS.agentk.md beside a team's own files."""
    if config.get("MAPS") == "beside":
        return "AGENTS" + (config.get("DOC_SUFFIX") or ".agentk.md")
    return "AGENTS.md"


def is_doc_for(doc: str, path: str, config: dict[str, str]) -> bool:
    """A map, kit detail doc or README.md in the path's folder or a folder above it."""
    if posixpath.basename(doc) not in ("AGENTS.md", "README.md", unit_doc(config), map_file(config)):
        return False
    folder = posixpath.dirname(doc)
    return not folder or path.startswith(folder + "/")


def map_anchors(repo: Path, code: list[str], name: str = "AGENTS.md") -> list[tuple[str, str, str]]:
    """(map, line, path) for every path named by a map above a changed file."""
    maps = set()
    for path in code:
        folder = posixpath.dirname(path)
        while True:
            candidate = posixpath.join(folder, name) if folder else name
            if (repo / candidate).is_file():
                maps.add(candidate)
            if not folder:
                break
            folder = posixpath.dirname(folder)
    if not maps:
        return []
    out = subprocess.run(["sh", ".agents/kit/check.sh", "--anchors", *sorted(maps)], cwd=repo,
                         capture_output=True, text=True, timeout=30).stdout
    anchors = []
    for line in out.splitlines():
        parts = line.split("\t")
        if len(parts) == 3:
            anchors.append((parts[0], parts[1], posixpath.normpath(parts[2])))
    return anchors


def changes_shape(repo: Path, folder: str, path: str, kind: str) -> bool:
    """A file added to or removed from the folder itself, or a whole subfolder
    that is new or gone: what a map row for the folder may need to say."""
    parts = path[len(folder) + 1:].split("/")
    if len(parts) == 1:
        return True
    sub = posixpath.join(folder, parts[0])
    if kind == "D":
        return not (repo / sub).exists()
    return not git(repo, "ls-files", "--", sub).strip()


def docs_items(repo: Path, config: dict[str, str], changes: dict[str, str], reminded: set[str]) -> list[str]:
    code = sorted(path for path in changes
                  if not path.endswith(".md") and not skipped(path, config) and path not in reminded)
    docs = [path for path in changes if path.endswith(".md")]
    code = [path for path in code if not any(is_doc_for(doc, path, config) for doc in docs)]
    if not code:
        return []
    anchors = map_anchors(repo, code, map_file(config))
    interface = config.get("INTERFACE_GLOBS", "").split()
    items = []
    for path in code:
        reason = ""
        for map_path, line, anchor in anchors:
            if path == anchor or fnmatch.fnmatch(path, anchor):
                reason = f"`{path}` is named in {map_path} (line {line})"
                break
            if changes[path] in "AD" and path.startswith(anchor + "/") and (repo / anchor).is_dir() \
                    and changes_shape(repo, anchor, path, changes[path]):
                verb = "added to" if changes[path] == "A" else "removed from"
                reason = f"`{path}` was {verb} `{anchor}/`, which {map_path} names (line {line})"
                break
        if not reason and any(fnmatch.fnmatch(path, glob) or fnmatch.fnmatch(posixpath.basename(path), glob)
                              for glob in interface):
            reason = f"`{path}` is an interface file (INTERFACE_GLOBS); check the {unit_doc(config)} that documents it"
        if reason:
            items.append((path, reason))
    reminded.update(path for path, _ in items)
    return [f"- {reason}" for _, reason in items[:5]] + (
        [f"- and {len(items) - 5} more"] if len(items) > 5 else [])


def front_paths(doc: Path) -> list[str]:
    text = doc.read_text(errors="replace")
    match = re.match(r"---\n(.*?)\n---", text.replace("\r\n", "\n"), re.S)
    found = re.search(r"^source_paths:\s*\[?(.*?)\]?\s*$", match.group(1), re.M) if match else None
    return [p.strip(" '\"") for p in found.group(1).split(",") if p.strip(" '\"")] if found else []


def mark_stale_learning(repo: Path, config: dict[str, str], changes: dict[str, str], state_dir: Path) -> None:
    folder = repo / config.get("LEARNING_DIR", "learning_docs")
    if not config.get("LEARNING_DIR", "learning_docs") or not folder.is_dir():
        return
    code = [path for path in changes if not path.endswith(".md")]
    stale_file = state_dir.parent / "stale-learning"
    known = set(stale_file.read_text().split()) if stale_file.exists() else set()
    for doc in sorted(folder.rglob("*.md")):
        name = doc.relative_to(repo).as_posix()
        sources = [source.rstrip("/") for source in front_paths(doc)]
        if name not in known and any(path == s or path.startswith(s + "/") for s in sources for path in code):
            known.add(name)
    if known:
        stale_file.write_text("".join(f"{name}\n" for name in sorted(known)))


def repo_root(cwd: str) -> Path:
    found = subprocess.run(["git", "-C", cwd, "rev-parse", "--show-toplevel"],
                           capture_output=True, text=True).stdout.strip()
    return Path(found or cwd).resolve()


def main(argv: list[str]) -> None:
    raw = sys.stdin.read()
    event = json.loads(raw) if raw.strip() else {}
    repo = repo_root(event.get("cwd") or ".")
    if not (repo / ".agents" / "kit" / "check.sh").exists():
        return
    common = git(repo, "rev-parse", "--git-common-dir").strip()
    state_dir = (repo / common / "agent-kit" / "state").resolve()
    state_dir.mkdir(parents=True, exist_ok=True)
    for old in state_dir.glob("*.json"):  # sessions older than a week
        if old.stat().st_mtime < time.time() - 7 * 86400:
            old.unlink()
    session = re.sub(r"[^A-Za-z0-9_-]", "_", str(event.get("session_id") or "default"))[:80]
    state_file = state_dir / f"{session}.json"
    now = snapshot(repo)
    if "--baseline" in argv:
        state_file.write_text(json.dumps({"snapshot": now, "reminded": []}))
        return
    if event.get("stop_hook_active"):
        return
    state = json.loads(state_file.read_text()) if state_file.exists() else None
    if state is None:
        if not turn_used_change_tools(event.get("transcript_path")):
            state_file.write_text(json.dumps({"snapshot": now, "reminded": []}))
            return
        state = {"snapshot": {"head": now["head"], "files": {}}, "reminded": []}
    changes = changes_since(repo, state["snapshot"], now)
    reminded = set(state.get("reminded", []))
    if not changes:
        return
    config = read_config(repo)
    beside = config.get("MAPS") == "beside"
    if beside and any(posixpath.basename(path) in TEAM_NAMES + (map_file(config),) for path in changes):
        try:  # a map changed: refresh the local files that connect it to each agent
            subprocess.run(["sh", ".agents/kit/wire.sh", "--quiet"], cwd=repo,
                           capture_output=True, timeout=60)
        except (OSError, subprocess.TimeoutExpired):
            pass
    failures = failing_checks(repo, sorted(changes))
    if failures:  # keep the old snapshot, so the next turn checks these files again
        print(json.dumps({"decision": "block", "reason": CHECKS_FAILING.format(failures=failures)}))
        return
    mark_stale_learning(repo, config, changes, state_dir)
    if beside:
        team = sorted(path for path in changes if posixpath.basename(path) in TEAM_NAMES
                      and f"team:{path}" not in reminded
                      and git(repo, "ls-files", "--", path).strip())
        if team:
            reminded.update(f"team:{path}" for path in team)
            state_file.write_text(json.dumps({"snapshot": now, "reminded": sorted(reminded)}))
            print(json.dumps({"decision": "block", "reason": TEAM_FILE.format(
                files=", ".join(f"`{path}`" for path in team), map_file=map_file(config))}))
            return
    items = docs_items(repo, config, changes, reminded)
    state_file.write_text(json.dumps({"snapshot": now, "reminded": sorted(reminded)}))
    if items:
        print(json.dumps({"decision": "block", "reason": REMINDER.format(
            items="\n".join(items), unit_doc=unit_doc(config))}))


if __name__ == "__main__":
    try:
        main(sys.argv[1:])
    except Exception:  # noqa: BLE001 - a broken hook must never block the agent
        pass
