# build-scripts

These scripts are only useful for building the distributed bacon binaries hosted
on the deployment server, so that a CI job can fetch bacon instead of compiling
it. If you just want to install or build bacon for yourself, you don't need
anything here — see <https://dystroy.org/bacon>.

They may be run from anywhere; each resolves paths against the repo root.

## Building a release

A Mac builds every target on its own: zig cross-builds the Linux and Windows
binaries, and the Apple ones are native. Elsewhere the Apple binaries are
missing, and the release has to be assembled from two machines through a staging
server (`BACON_STAGE_HOST`, see below).

1. Commit and push, then on **each** machine that builds (just the Mac, unless
   you've set up staging):

       ./build-scripts/build-all-targets.sh

   Builds this host's targets. With staging configured it also pushes `build/` to
   the staging server, keyed by `<version>-<commit>`; targets already staged under
   that key are skipped, so a re-run after a failure only builds what's missing,
   and `--force` rebuilds them all. Any new commit changes the key, so everything
   is rebuilt. It warns when HEAD isn't `origin/main`.

   Without staging, `release.sh` runs it itself, so on a lone Mac this step can
   be skipped.

2. On one machine, assemble and package:

       ./build-scripts/release.sh

   Refuses to go on unless HEAD is exactly `origin/main` (not unpushed, not
   behind) and no `v<version>` tag exists on origin at another commit. Then
   fetches the staged artifacts (or, without staging, builds locally, which only
   a Mac can do completely), checks every target is present, verifies each
   binary, and produces `bacon_<version>.zip` and `dist/`, the files of the
   GitHub release (see [The archives](#the-archives)).

   `--dry-run` packages without committing to a release, to inspect `dist/`:
   it neither stops nor asks, but lists at the end everything a real release
   would stop on or ask about (exiting with 1 if it would stop).

3. Publish:

       ./build-scripts/deploy.sh

   rsyncs `build/` and the zip straight into the download directory on the
   server. Nothing goes through `~/dev/www/dystroy`: that tree mirrors the whole
   site on each machine, so pushing it from one machine republishes its stale
   copy of everything the other machines deployed. Previous zips are left in
   place, so old versions stay downloadable.

   It then creates and pushes the `v<version>` tag on HEAD (retrying when GitHub
   fails transiently), runs `github-release.sh`, and runs `website/deploy.sh`,
   last, so the site never describes a version whose binaries aren't online yet.

   `github-release.sh` creates the GitHub release from that tag as a **draft**,
   with the files of `dist/` and the version's `CHANGELOG.md` section as notes;
   when the release already exists, its files are replaced instead. If it fails,
   `deploy.sh` warns and goes on with the website: fix the cause, then run
   `./build-scripts/github-release.sh` alone. It refuses a dry run's `dist/`.

   Both need `gh`, logged in to github.com (`gh auth login`); `deploy.sh` checks
   it before building.

4. Review the draft on GitHub, edit its notes if needed, and publish it.

Staging and deploy settings come from `build-scripts/_local.sh` (see below).
Without it, `release.sh` builds locally on a single host, which must be a Mac,
and `deploy.sh` won't run.

A dirty tree doesn't block either step, but both ask first: `build-all-targets.sh`
offers to stage it anyway (recorded in a `<version>-<commit>.dirty` marker beside
the staging dir, and already-staged targets are then rebuilt rather than reused),
and `release.sh` reports every host that staged uncommitted work before packaging.
Set `BACON_YES=1` to answer yes to all of it without a terminal.

## The archives

`bacon_<version>.zip` holds the contents of `build/`, with no wrapping directory:

    aarch64-apple-darwin/bacon       one directory per triple
    x86_64-pc-windows-gnu/bacon.exe
    ...
    CHANGELOG.md
    README.md
    install.md
    version                          read by the website to show the current version

`dist/` holds exactly the files of the GitHub release: that zip, one archive per
target, and `sha256sums`. Each per-target archive is a `.tar.gz` (a `.zip` for
Windows) unpacking to a single directory:

    bacon-v<version>-<triple>/
        bacon                        bacon.exe on Windows
        CHANGELOG.md
        LICENSE
        README.md

## Targets

The release matrix lives in `_targets.sh`. Linux and Windows are built with
`cargo-zigbuild`, which needs `zig` but no container engine; the two Apple
targets are built natively on a Mac, because zig produces macOS binaries with
duplicate linked dylibs.

| Label | Triple | Tool | Features |
|-------|--------|------|----------|
| x86-64 GLIBC | `x86_64-unknown-linux-gnu` | zig | clipboard |
| MUSL | `x86_64-unknown-linux-musl` | zig | clipboard |
| ARM 64 | `aarch64-unknown-linux-gnu` | zig | clipboard |
| ARM 64 MUSL | `aarch64-unknown-linux-musl` | zig | clipboard |
| Windows | `x86_64-pc-windows-gnu` | zig | clipboard, sound |
| macOS ARM | `aarch64-apple-darwin` | native (Mac only) | clipboard, sound |
| macOS Intel | `x86_64-apple-darwin` | native (Mac only) | clipboard, sound |

The Linux targets are built without `sound`: alsa would be needed on the build
host, and the binary would then require `libasound.so.2` on every machine
running it, which a CI container usually lacks. Windows and macOS get it, their
audio API being part of the system. Linux users wanting sound install with
`cargo install --locked bacon --features "clipboard sound"`.

`DARWIN_METHOD` overrides how the Apple targets are handled: `auto` (default,
native on a Mac and skipped elsewhere), `native`, or `skip`.

Every built binary is checked: the arch `file` reports must match the triple,
musl binaries are expected to be statically linked, macOS binaries must have no
duplicate linked dylib, and a binary built for the host triple is run to confirm
it reports the current version.

## Machine-local config (`_local.sh`)

`build-scripts/_local.sh` holds per-machine settings and is **gitignored**, so it
never travels through git — **recreate it on each machine** that builds or deploys.
It's sourced by `_common.sh`.

| Variable | Used by | Required | Meaning |
|----------|---------|----------|---------|
| `BACON_STAGE_HOST` | build-all-targets.sh, release.sh | for staged releases | ssh host every machine can reach; enables push/fetch of a multi-host release. Unset ⇒ single-host local builds. |
| `BACON_STAGE_DIR` | build-all-targets.sh, release.sh | no — default `bacon-staging` | staging dir on the server, relative to your ssh login home (or absolute, with a leading `/`). |
| `BACON_RELEASE_BRANCH` | build-all-targets.sh, release.sh | no — default `main` | branch of origin the released commit must be. |
| `BACON_DEPLOY_TARGET` | deploy.sh | yes, to deploy | rsync destination of the download dir, e.g. `dys@dystroy.org:prod/www.dystroy.org/bacon/download`. Must end in `/bacon/download`. |

A full example:

```bash
# build-scripts/_local.sh  — per machine, gitignored

# Staged multi-host release builds (only when several machines build):
BACON_STAGE_HOST=dystroy.org
BACON_STAGE_DIR=staging/bacon-staging        # relative to ssh home, or absolute

# Publishing, on whichever machine runs deploy.sh:
BACON_DEPLOY_TARGET="dys@dystroy.org:prod/www.dystroy.org/bacon/download"
```

## Other scripts

- `build-target.sh <filter>` — build one target (`--list` to see them), e.g.
  `./build-scripts/build-target.sh aarch64-apple-darwin`
- `github-release.sh` — create or update the draft GitHub release from `dist/`;
  run by `deploy.sh`, run it alone after fixing a failure there
- `_common.sh`, `_targets.sh` — sourced libraries, not run directly
