#!/usr/bin/env bash
#
# Assemble and package a full bacon release: build/ plus bacon_<version>.zip in
# the repo root (the zip is both a download and the artifact to attach to the
# GitHub release).
#
# With staging configured (BACON_STAGE_HOST, see _local.sh) this does NOT build —
# it fetches the artifacts every host pushed for the current commit and checks
# the set is complete (so the macOS binaries from the Mac and the linux/windows
# ones from wherever they were built come together). Without staging it falls
# back to a local build-all (single host). Either way it then verifies every
# binary and zips.

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version=$(bacon_version)

# Publishing a binary built from uncommitted work is worth a question, whether
# or not the artifacts travelled through a staging server.
if ! tree_is_clean; then
    warn "working tree has uncommitted changes, so this release won't match commit $(git rev-parse --short HEAD)"
    confirm "assemble it anyway?" || die "aborted — commit your changes, then re-run"
fi

if staging_configured; then
    id=$(release_id)
    h1 "Assembling release $version from $BACON_STAGE_HOST ($id)"
    rm -rf build && mkdir build
    h2 "Fetching staged artifacts"
    stage_fetch build
    ok "fetched $BACON_STAGE_DIR/$id"
    # A host may have staged from a dirty tree — including a host that isn't this
    # one, so a clean tree here is no guarantee about the artifacts.
    dirty_report=$(stage_dirty_report)
    if [[ -n $dirty_report ]]; then
        warn "some staged artifacts were built from uncommitted changes:"
        while IFS= read -r line; do
            [[ -n $line ]] && info "$line"
        done <<< "$dirty_report"
        confirm "package them anyway?" || die "aborted — rebuild those targets from a clean tree"
    fi
    manifest_fn=all_release_targets   # a full release must contain every target
else
    h1 "Building release $version locally — THIS host's targets only"
    # A Mac covers the whole matrix (zig for linux/windows, native for Apple);
    # elsewhere the Apple targets are missing, and that's worth saying.
    if [[ $(all_targets | wc -l) -lt $(all_release_targets | wc -l) ]]; then
        warn "this host can't build every release target (no Apple binary outside a Mac)."
        warn "to assemble a full release, set BACON_STAGE_HOST in build-scripts/_local.sh on each machine."
    fi
    "$here/build-all-targets.sh"
    manifest_fn=all_targets           # can only expect what this host builds
fi

# Completeness: every target of the full (cross-host) release manifest must be
# present, and each binary is verified (arch, and no duplicate dylib on macOS).
h2 "Checking release completeness"
missing=()
while IFS='|' read -r label triple tool features; do
    bin=$(target_binary "$triple")
    if [[ -f $bin ]]; then
        verify_binary "$bin" "$triple"
    else
        missing+=("$label ($triple)")
    fi
done < <("$manifest_fn")
[[ ${#missing[@]} -eq 0 ]] || die "release incomplete — missing binaries: ${missing[*]}"
ok "all release targets present and verified"

# The non-binary artifacts must be there too.
for f in README.md CHANGELOG.md version install.md; do
    [[ -e "build/$f" ]] || die "release artifact missing from build/: $f"
done
ok "metadata present (version, changelog, readme)"

# build the release archive
rm -f bacon_*.zip
( cd build && zip -rq "../bacon_$version.zip" -- * )
ok "created bacon_$version.zip"

h1 "Release $version ready: bacon_$version.zip"
