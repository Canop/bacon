#!/usr/bin/env bash
#
# Assemble and package a full bacon release: build/, bacon_<version>.zip in the
# repo root (the download served on dystroy.org), and dist/, which holds exactly
# the files to attach to the GitHub release: one archive per target, the global
# zip, and sha256sums.
#
# With staging configured (BACON_STAGE_HOST, see _local.sh) this does NOT build —
# it fetches the artifacts every host pushed for the current commit and checks
# the set is complete (so the macOS binaries from the Mac and the linux/windows
# ones from wherever they were built come together). Without staging it falls
# back to a local build-all (single host). Either way it then verifies every
# binary and packages.
#
# --dry-run packages for inspection: it neither stops nor asks, but lists at the
# end everything a real release would stop on or ask about. It exits with 1 when
# a real release would stop.

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

dry_run=no
case ${1:-} in
    "") ;;
    --dry-run|-n) dry_run=yes ;;
    *) die "usage: $(basename "$0") [--dry-run]" ;;
esac
[[ $# -le 1 ]] || die "usage: $(basename "$0") [--dry-run]"

stops=()
asks=()
# Die, or in a dry run note it and go on.
stop() { # stop <message>
    [[ $dry_run == yes ]] || die "$1"
    warn "$1"
    stops+=("$1")
}
# Ask, or in a dry run note the question and go on.
ask() { # ask <question> <abort message>
    if [[ $dry_run == yes ]]; then
        asks+=("$1")
        return 0
    fi
    confirm "$1" || die "$2"
}

version=$(bacon_version)
[[ $dry_run == yes ]] && h1 "DRY RUN — packaging $version for inspection"

# The GitHub release and the source archives come from origin, so the binaries
# must be built from the commit that's there.
h2 "Checking the release source"
if [[ $dry_run == yes ]]; then
    # A separate process, so that errexit applies inside the check, and no ERR
    # trap here, as bash 3.2 runs it even in a condition.
    if out=$(trap - ERR; "$BASH" -c 'source "$1"; check_release_source' _ "$here/_common.sh" 2>&1); then
        printf '%s\n' "$out"
    else
        reason=
        while IFS= read -r line; do
            if [[ $line == "✗ "* ]]; then
                reason=${line#"✗ "}
                break
            fi
        done <<< "$out"
        stop "${reason:-the release source check failed}"
    fi
else
    check_release_source
fi

# Publishing a binary built from uncommitted work is worth a question, whether
# or not the artifacts travelled through a staging server.
if ! tree_is_clean; then
    warn "working tree has uncommitted changes, so this release won't match commit $(git rev-parse --short HEAD)"
    ask "uncommitted changes — assemble it anyway?" "aborted — commit your changes, then re-run"
fi

if staging_configured; then
    id=$(release_id)
    h1 "Assembling release $version from $BACON_STAGE_HOST ($id)"
    staged=$(stage_has_release) || die "couldn't reach $BACON_STAGE_HOST"
    [[ $staged == yes ]] || die "nothing staged for $id — run build-all-targets.sh first"
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
        ask "staged artifacts built from uncommitted changes — package them anyway?" \
            "aborted — rebuild those targets from a clean tree"
    fi
else
    h1 "Building release $version locally"
    # A Mac covers the whole matrix (zig for linux/windows, native for Apple);
    # elsewhere the Apple binaries are missing, which only a dry run accepts.
    if [[ $(all_targets | wc -l) -lt $(all_release_targets | wc -l) ]]; then
        hint="this host can't build every release target (no Apple binary outside a Mac) — set BACON_STAGE_HOST in build-scripts/_local.sh on each machine"
        [[ $dry_run == yes ]] || die "$hint"
        warn "$hint"
    fi
    "$here/build-all-targets.sh"
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
done < <(all_release_targets)
if [[ ${#missing[@]} -gt 0 ]]; then
    stop "release incomplete — missing binaries: ${missing[*]}"
else
    ok "all release targets present and verified"
fi

# The non-binary artifacts must be there too.
missing=()
for f in README.md CHANGELOG.md version install.md; do
    [[ -e "build/$f" ]] || missing+=("$f")
done
if [[ ${#missing[@]} -gt 0 ]]; then
    stop "release artifacts missing from build/: ${missing[*]}"
else
    ok "metadata present (version, changelog, readme)"
fi

# build the release archive, whose entries are the contents of build/, not a
# build/ directory: bacon_3.26.0.zip unzips to aarch64-apple-darwin/, README.md, etc.
rm -f bacon_*.zip
( cd build && zip -rq "../bacon_$version.zip" -- * )
ok "created bacon_$version.zip"

# One archive per target, each unpacking to a single bacon-v<version>-<triple>/
# directory: a .zip for Windows, a .tar.gz elsewhere.
h2 "Packaging one archive per target in dist/"
rm -rf dist && mkdir -p dist/.tmp
while IFS='|' read -r label triple _tool _features; do
    bin=$(target_binary "$triple")
    [[ -f $bin ]] || continue
    name="$NAME-v$version-${triple%%.*}"
    mkdir "dist/.tmp/$name"
    cp "$bin" "dist/.tmp/$name/"
    for f in build/README.md build/CHANGELOG.md LICENSE; do
        if [[ -e $f ]]; then
            cp -R "$f" "dist/.tmp/$name/"
        fi
    done
    if [[ $triple == *windows* ]]; then
        ( cd dist/.tmp && zip -rqX "../$name.zip" "$name" )
        ok "dist/$name.zip"
    else
        # keep macOS metadata (._ files, xattrs) out of the archive
        COPYFILE_DISABLE=1 tar --no-xattrs -czf "dist/$name.tar.gz" -C dist/.tmp "$name"
        ok "dist/$name.tar.gz"
    fi
done < <(all_release_targets)
rm -rf dist/.tmp
cp "bacon_$version.zip" dist/
sha256() { if have sha256sum; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
( cd dist && sha256 *.tar.gz *.zip > sha256sums )
ok "dist/sha256sums"

if [[ $dry_run == yes ]]; then
    # github-release.sh refuses to publish a dry run's dist/
    touch dist/.dry-run
    h1 "DRY RUN done — inspect dist/"
    if [[ ${#stops[@]} -gt 0 ]]; then
        warn "a real release would stop:"
        for s in "${stops[@]}"; do info "- $s"; done
    fi
    if [[ ${#asks[@]} -gt 0 ]]; then
        warn "a real release would ask:"
        for a in "${asks[@]}"; do info "- $a"; done
    fi
    [[ ${#stops[@]} -eq 0 ]] || exit 1
    if [[ ${#asks[@]} -gt 0 ]]; then
        ok "a real release would go through, once those questions are answered"
    else
        ok "a real release would go through"
    fi
    exit 0
fi

h1 "Release $version ready: bacon_$version.zip and dist/"
info "deploy.sh publishes it, tags $(release_tag), and creates the GitHub release as a draft"
