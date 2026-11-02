#!/usr/bin/env bash
#
# Create the GitHub release of the current version as a draft, with the files
# of dist/ and the version's CHANGELOG.md section as notes. When the release
# already exists, its files are replaced instead.
#
# Run by deploy.sh after the tag is pushed; run it again by hand if it failed
# there. It needs the dist/ of a real (not dry) release.sh run.

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

need_gh

version=$(bacon_version)
tag=$(release_tag)

[[ -f "dist/bacon_$version.zip" ]] || die "no dist/ for $version — run release.sh first"
[[ ! -e dist/.dry-run ]] || die "dist/ comes from a dry run — run release.sh without --dry-run"
assets=(dist/*.tar.gz dist/*.zip dist/sha256sums)

h2 "Creating the GitHub release $tag"
tags=$(gh release list --repo "$BACON_GITHUB_REPO" --limit 1000 --json tagName --jq '.[].tagName')
if grep -qxF "$tag" <<< "$tags"; then
    gh release upload "$tag" "${assets[@]}" --repo "$BACON_GITHUB_REPO" --clobber
    ok "files of the existing release $tag replaced"
else
    notes=$(mktemp)
    changelog_notes "$version" > "$notes"
    [[ -s $notes ]] || warn "no section for v$version in CHANGELOG.md, the release notes are empty"
    gh release create "$tag" "${assets[@]}" --repo "$BACON_GITHUB_REPO" \
        --verify-tag --draft --title "bacon $tag" --notes-file "$notes"
    rm -f "$notes"
    ok "draft release $tag created with ${#assets[@]} files — review and publish it on GitHub"
fi
