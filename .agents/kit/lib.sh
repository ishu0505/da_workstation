# Shared helpers for the kit's scripts. Source it from the repo root:
#   . ./.agents/kit/lib.sh && load_conf
# Installed by agent_dev_kit; do not edit here.

# load_conf [file]: read the KEY="value" lines of the config (default
# .agents/agent-kit.conf) into shell variables. The file is read as text,
# never run as shell code, so a value with $ or ` stays exactly as written.
# Tolerates CRLF line endings. A missing file changes nothing.
load_conf() {
  _conf_file=${1:-.agents/agent-kit.conf}
  [ -f "$_conf_file" ] || return 0
  _conf_lines=$(tr -d '\r' < "$_conf_file" |
    sed -n 's/^[[:space:]]*\([A-Z_][A-Z0-9_]*\)="\{0,1\}\([^"]*\)"\{0,1\}[[:space:]]*$/\1=\2/p')
  while IFS= read -r _conf_line; do
    [ -n "$_conf_line" ] || continue
    _conf_key=${_conf_line%%=*}
    _conf_value=${_conf_line#*=}
    eval "$_conf_key=\$_conf_value"  # the key is [A-Z_0-9]; the value is assigned, not run
  done <<EOF
$_conf_lines
EOF
}

# is_code <path>: a code file (CODE_EXTENSIONS) outside skipped folders.
# A folder is skipped when its name starts with . or _, is in SKIP_DIRS, or
# is LEARNING_DIR. Only folders are tested this way: __init__.py and
# _app.tsx are code. A file name that starts with a dot is not.
is_code() {
  case "$1" in */*) _code_rest=${1%/*} ;; *) _code_rest="" ;; esac
  while [ -n "$_code_rest" ]; do
    _code_part=${_code_rest%%/*}
    case "$_code_part" in .*|_*) return 1 ;; esac
    [ "$_code_part" = "${LEARNING_DIR:-}" ] && return 1
    for _code_skip in ${SKIP_DIRS:-}; do [ "$_code_part" = "$_code_skip" ] && return 1; done
    case "$_code_rest" in */*) _code_rest=${_code_rest#*/} ;; *) _code_rest="" ;; esac
  done
  _code_name=${1##*/}
  case "$_code_name" in .*) return 1 ;; esac
  _code_ext=${_code_name##*.}
  [ "$_code_ext" != "$_code_name" ] || return 1
  case " ${CODE_EXTENSIONS:-} " in *" $_code_ext "*) return 0 ;; esac
  return 1
}
