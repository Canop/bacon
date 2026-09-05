# build-scripts

These scripts are only useful for building the distributed bacon binaries hosted
on the deployment server, so that a CI job can fetch bacon instead of compiling
it. If you just want to install or build bacon for yourself, you don't need
anything here — see <https://dystroy.org/bacon>.

They may be run from anywhere; each resolves paths against the repo root.

## Building a release

No single machine can build every target (the macOS binaries need a Mac), so a
release is built on both a Mac and a Linux box and assembled through a staging
server.

1. Commit, then on **each** machine (Mac and Linux):

       ./build-scripts/build-all-targets.sh

   Builds this host's targets and pushes `build/` to the staging server, keyed by
   `<version>-<commit>`. Targets already staged under that key are skipped, so a
   re-run after a failure only builds what's missing; `--force` rebuilds them all.
   Any new commit changes the key, so everything is rebuilt.

2. On one machine, assemble and package:

       ./build-scripts/release.sh

   Fetches the staged artifacts, checks every target is present, verifies each
   binary, and produces `bacon_<version>.zip` (also the artifact to attach to the
   GitHub release).

3. Publish:

       ./build-scripts/deploy.sh

   rsyncs `build/` and the zip straight into the download directory on the
   server. Nothing goes through `~/dev/www/dystroy`: that tree mirrors the whole
   site on each machine, so pushing it from one machine republishes its stale
   copy of everything the other machines deployed. Previous zips are left in
   place, so old versions stay downloadable.

Staging and deploy settings come from `build-scripts/_local.sh` (see below).
Without it, `release.sh` builds locally on a single host and `deploy.sh` won't run.

A dirty tree doesn't block either step, but both ask first: `build-all-targets.sh`
offers to stage it anyway (recorded in a `<version>-<commit>.dirty` marker beside
the staging dir, and already-staged targets are then rebuilt rather than reused),
and `release.sh` reports every host that staged uncommitted work before packaging.
Set `BACON_YES=1` to answer yes to all of it without a terminal.

## Targets

The release matrix lives in `_targets.sh`. Linux and Windows are built with
`cargo-zigbuild`, which needs `zig` but no container engine; the two Apple
targets are built natively on a Mac, because zig produces macOS binaries with
duplicate linked dylibs.

| Label | Triple | Tool |
|-------|--------|------|
| x86-64 GLIBC | `x86_64-unknown-linux-gnu` | zig |
| MUSL | `x86_64-unknown-linux-musl` | zig |
| ARM 64 | `aarch64-unknown-linux-gnu` | zig |
| ARM 64 MUSL | `aarch64-unknown-linux-musl` | zig |
| Windows | `x86_64-pc-windows-gnu` | zig |
| macOS ARM | `aarch64-apple-darwin` | native (Mac only) |
| macOS Intel | `x86_64-apple-darwin` | native (Mac only) |

All are built with the `clipboard` feature and without `sound`, which would
require alsa on the build and run hosts.

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
| `BACON_DEPLOY_TARGET` | deploy.sh | yes, to deploy | rsync destination of the download dir, e.g. `dys@dystroy.org:prod/www.dystroy.org/bacon/download`. Must end in `/bacon/download`. |

A full example:

```bash
# build-scripts/_local.sh  — per machine, gitignored

# Staged multi-host release builds (set on both the Mac and the Linux box):
BACON_STAGE_HOST=dystroy.org
BACON_STAGE_DIR=staging/bacon-staging        # relative to ssh home, or absolute

# Publishing, on whichever machine runs deploy.sh:
BACON_DEPLOY_TARGET="dys@dystroy.org:prod/www.dystroy.org/bacon/download"
```

## Other scripts

- `build-target.sh <filter>` — build one target (`--list` to see them), e.g.
  `./build-scripts/build-target.sh aarch64-apple-darwin`
- `_common.sh`, `_targets.sh` — sourced libraries, not run directly
