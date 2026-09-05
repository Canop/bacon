# _common.sh — shared helpers for bacon's build/release/deploy scripts.
#
# Source it near the top of a script:
#   source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
# It enables strict mode and an error trap for the sourcing script.
#
# This file is not meant to be run directly.

set -Eeuo pipefail

# --- error reporting ---------------------------------------------------------
_bacon_on_err() {
    local code=$?
    printf '\n%s✗ failed (exit %d)%s at %s:%s\n    %s\n' \
        "${_c_err:-}" "$code" "${_c_reset:-}" \
        "${BASH_SOURCE[1]:-?}" "${BASH_LINENO[0]:-?}" "$BASH_COMMAND" >&2
    exit "$code"
}
trap _bacon_on_err ERR

# Resolve all relative paths against the repo root. This file lives in
# build-scripts/, so the root is its parent directory.
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# --- decorations (only when writing to a terminal, and NO_COLOR unset) -------
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
    _c_reset=$'\033[0m'; _c_h1=$'\033[1;97;44m'; _c_h2=$'\033[97;44m'
    _c_ok=$'\033[32m'; _c_warn=$'\033[33m'; _c_err=$'\033[31m'
else
    _c_reset=; _c_h1=; _c_h2=; _c_ok=; _c_warn=; _c_err=
fi
h1()   { printf '\n%s %s %s\n' "$_c_h1" "$*" "$_c_reset"; }
h2()   { printf '\n%s %s %s\n' "$_c_h2" "$*" "$_c_reset"; }
info() { printf '    %s\n' "$*"; }
ok()   { printf '    %s✓%s %s\n' "$_c_ok" "$_c_reset" "$*"; }
warn() { printf '%s⚠ %s%s\n' "$_c_warn" "$*" "$_c_reset" >&2; }
die()  { printf '%s✗ %s%s\n' "$_c_err" "$*" "$_c_reset" >&2; exit 1; }

# --- tool detection ----------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }
need() { # need <tool> [install hint]
    have "$1" && return 0
    die "required tool '$1' not found${2:+ — $2}"
}

# --- facts about the project / host ------------------------------------------
# First matching line only, without piping into head (which would close the
# pipe early and, under pipefail, abort the script on SIGPIPE).
bacon_version() {
    local v
    v=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' Cargo.toml)
    printf '%s\n' "${v%%$'\n'*}"
}
host_target() {
    local t
    t=$(rustc -vV | sed -n 's/^host: //p')
    printf '%s\n' "${t%%$'\n'*}"
}
host_os() { uname -s; }

# --- portability helpers -----------------------------------------------------
safe_wipe() { # remove the *contents* of a directory, refusing dangerous targets
    local dir=${1:-}
    [[ -n "$dir" ]] || die "safe_wipe: refusing to wipe an empty path"
    [[ "$dir" != "/" && "$dir" != "$HOME" ]] || die "safe_wipe: refusing to wipe '$dir'"
    [[ -d "$dir" ]] && rm -rf "${dir:?}"/*
    return 0
}

# --- release staging (optional multi-host builds via a shared server) ---------
# When BACON_STAGE_HOST is set (see _local.sh), build-all-targets.sh pushes its
# build/ to <host>:<BACON_STAGE_DIR>/<id>/ and release.sh fetches the union from
# there. <id> ties every artifact to one commit. When unset, builds stay local.
staging_configured() { [[ -n ${BACON_STAGE_HOST:-} ]]; }

tree_is_clean() { git diff --quiet HEAD 2>/dev/null; }

# Ask a yes/no question on the terminal, defaulting to no. With no terminal to
# ask on (cron, CI) the answer is no, unless BACON_YES is set.
confirm() { # confirm <question>
    local reply
    if [[ -n ${BACON_YES:-} ]]; then
        info "$1 — assuming yes (BACON_YES is set)"
        return 0
    fi
    printf '    %s [y/N] ' "$1"
    { read -r reply </dev/tty; } 2>/dev/null || reply=n
    case $reply in [yY] | [yY][eE][sS]) return 0 ;; *) return 1 ;; esac
}

release_id() { # <version>-<short commit>, e.g. 3.25.0-813af70
    printf '%s-%s\n' "$(bacon_version)" "$(git rev-parse --short HEAD)"
}

stage_push() { # stage_push <local-dir>  -> pushes its contents into <dir>/<id>/
    local dir=$1 id dest
    id=$(release_id)
    dest="$BACON_STAGE_DIR/$id"
    ssh "$BACON_STAGE_HOST" "mkdir -p '$dest'"
    rsync -az "$dir"/ "$BACON_STAGE_HOST:$dest/"
}

stage_fetch() { # stage_fetch <local-dir>  <- pulls <dir>/<id>/ into it
    local dir=$1 id
    id=$(release_id)
    rsync -az "$BACON_STAGE_HOST:$BACON_STAGE_DIR/$id/" "$dir"/
}

# A build staged from a dirty tree doesn't match its commit, so the host that
# staged it says so in a marker file. It sits *beside* the staging dir, not in
# it, so it never travels into build/ or the release zip.
stage_dirty_marker() { printf '%s/%s.dirty\n' "$BACON_STAGE_DIR" "$(release_id)"; }

stage_mark_dirty() {
    local line
    line="$(date -u +%Y-%m-%dT%H:%M:%SZ) $(hostname) $(host_os)"
    ssh "$BACON_STAGE_HOST" \
        "mkdir -p '$BACON_STAGE_DIR' && echo '$line' >> '$(stage_dirty_marker)'"
}

# One line per host that staged uncommitted work for this id; empty if none.
stage_dirty_report() {
    ssh "$BACON_STAGE_HOST" "cat '$(stage_dirty_marker)' 2>/dev/null" 2>/dev/null || true
}

# --- machine-local overrides (gitignored): staging + deploy paths -------------
_bacon_common_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ -f "$_bacon_common_dir/_local.sh" ]] && source "$_bacon_common_dir/_local.sh"
: "${BACON_STAGE_HOST:=}"
: "${BACON_STAGE_DIR:=bacon-staging}"
