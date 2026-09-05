#!/usr/bin/env bash
#
# Build every distributed bacon binary and assemble the build/ directory
# (binaries + release metadata) ready for release.sh.
#
# This is NOT for normal installation (see https://dystroy.org/bacon); it's the
# multi-toolchain build used to produce the binaries hosted for download.
# It takes no arguments.
#
# Targets already staged for the current commit are skipped; pass --force to
# rebuild them anyway.
#
# To build just one target (e.g. to check a change compiles), use:
#   ./build-target.sh <target>

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"

force=no
case ${1:-} in
    "")         ;;
    --force|-f) force=yes ;;
    *) die "usage: $(basename "$0") [--force]; use ./build-target.sh <target> to build one" ;;
esac
[[ $# -le 1 ]] || die "usage: $(basename "$0") [--force]"

version=$(bacon_version)
h1 "Building all targets for $NAME $version   (host $(host_os), darwin=$DARWIN_METHOD)"
[[ $DARWIN_METHOD == skip ]] && warn "the macOS binaries will not be built (DARWIN_METHOD=skip)"

# Decide up front whether this build will be staged, so a dirty tree is settled
# now rather than after a long build.
stage_this=no
stage_dirty=no
if staging_configured; then
    if tree_is_clean; then
        stage_this=yes
    else
        warn "working tree has uncommitted changes, so this build won't match commit $(git rev-parse --short HEAD)"
        info "s = stage it anyway, marked dirty (release.sh will warn before packaging)"
        info "l = build locally, without staging"
        info "anything else = abort"
        if [[ -n ${BACON_YES:-} ]]; then
            info "BACON_YES is set — staging it anyway"
            reply=s
        else
            printf '    stage, local, or abort? [s/l/N] '
            { read -r reply </dev/tty; } 2>/dev/null || reply=n
        fi
        case $reply in
            [sS]) stage_this=yes; stage_dirty=yes ;;
            [lL]) ;;
            *) die "aborted — commit your changes, then re-run to stage the build" ;;
        esac
    fi
fi

# Anything already staged under this release id was built from this very commit,
# so it doesn't need rebuilding. Only consider it when this build would itself be
# staged (staging configured + clean tree): with a dirty tree the local source no
# longer matches what was staged, so reusing it would be wrong.
staged=()
if [[ $stage_this == yes && $stage_dirty == no && $force == no ]]; then
    h2 "Checking what is already staged"
    while IFS= read -r t; do
        [[ -n $t ]] && staged+=("$t")
    done < <(staged_triples)
    if [[ ${#staged[@]} -gt 0 ]]; then
        ok "${#staged[@]} target(s) already staged for $(release_id), they'll be skipped"
    else
        info "nothing staged yet for $(release_id)"
    fi
fi

h2 "Cleaning build/"
rm -rf build && mkdir build
ok "build/ cleaned"

built=0
skipped=0
while IFS= read -r row; do
    [[ -n $row ]] || continue
    IFS='|' read -r label triple _tool _feat <<< "$row"
    if [[ ${#staged[@]} -gt 0 ]] && printf '%s\n' "${staged[@]}" | grep -qxF "$triple"; then
        info "$label ($triple) already staged, skipped"
        skipped=$((skipped + 1))
        continue
    fi
    build_row "$row"
    built=$((built + 1))
done < <(all_targets)
ok "$built binaries built, $skipped skipped"

# Release metadata. Host-independent and deterministic for a given version, so
# it's safe for several hosts to stage it into the same <id>/.
h2 "Adding release metadata"
echo "This is bacon. More info and installation instructions on https://dystroy.org/bacon" > build/README.md
cp CHANGELOG.md build/
printf 'This archive contains pre-compiled binaries.\n\nFor more information, or if you prefer to compile yourself, see https://dystroy.org/bacon\n' > build/install.md
echo "$version" > build/version
ok "metadata added"

# Publish the whole build/ to the staging server (the decision was made up front,
# before the build, so a dirty tree is caught early rather than after all this work).
if [[ $stage_this == yes ]]; then
    h2 "Staging build/ to $BACON_STAGE_HOST:$BACON_STAGE_DIR/$(release_id)"
    stage_push build
    if [[ $stage_dirty == yes ]]; then
        stage_mark_dirty
        warn "staged as $(release_id) from a DIRTY tree — recorded in $(stage_dirty_marker)"
    else
        ok "staged as $(release_id)"
    fi
fi

h1 "FINISHED — build/ is ready for release.sh"
