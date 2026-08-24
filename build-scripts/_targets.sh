# _targets.sh — the list of release targets and the per-target build engine,
# shared by build-all-targets.sh and build-target.sh.
#
# Not meant to be run directly; source it:
#   source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

NAME=bacon

# Each target builds into its own cargo target dir under here, so native and
# zig builds never share host artifacts (and keep incremental caching).
CACHE=.build-cache

# How to build the macOS binaries:
#   auto   -> native when running on macOS, else skip
#   native -> native cargo build (must run on a Mac; zig can't be used for Apple
#             targets, it produces binaries with duplicate linked dylibs)
#   skip   -> don't build the macOS binaries
DARWIN_METHOD=${DARWIN_METHOD:-auto}
if [[ $DARWIN_METHOD == auto ]]; then
    if [[ $(host_os) == Darwin ]]; then DARWIN_METHOD=native; else DARWIN_METHOD=skip; fi
fi

# The release matrix, one "label|triple|tool|features|host" row per line.
#   tool: zig | native
#   host: optional — restrict to a host OS (uname -s, e.g. Linux, Darwin). Empty = any.
#   Lines that are blank or start with # are ignored — comment a row out to disable a target.
# macOS is not listed here; it's handled separately via DARWIN_METHOD.
#
# A triple may carry a glibc suffix (e.g. x86_64-unknown-linux-gnu.2.28) to pin
# the minimum glibc; zigbuild honours it and the suffix is dropped from the
# artifact paths. Not used currently: the musl binaries are the portable ones.
_matrix_rows() {
    while IFS= read -r line; do
        [[ $line =~ ^[[:space:]]*(#|$) ]] && continue
        printf '%s\n' "$line"
    done <<'EOF'
x86-64 GLIBC|x86_64-unknown-linux-gnu|zig|clipboard|
MUSL|x86_64-unknown-linux-musl|zig|clipboard|
ARM 64|aarch64-unknown-linux-gnu|zig|clipboard|
ARM 64 MUSL|aarch64-unknown-linux-musl|zig|clipboard|
Windows|x86_64-pc-windows-gnu|zig|clipboard|
EOF
}

# The macOS rows for the current DARWIN_METHOD (empty when skipped).
_darwin_rows() {
    case $DARWIN_METHOD in
        native)
            echo "macOS ARM|aarch64-apple-darwin|native|clipboard"
            echo "macOS Intel|x86_64-apple-darwin|native|clipboard" ;;
        skip) : ;;
    esac
}

# Targets to build on THIS host: matrix rows whose host matches (or is empty),
# plus the macOS rows per DARWIN_METHOD. Emits "label|triple|tool|features".
all_targets() {
    local host label triple tool features want
    host=$(host_os)
    _matrix_rows | while IFS='|' read -r label triple tool features want; do
        [[ -z $want || $want == "$host" ]] || continue
        printf '%s|%s|%s|%s\n' "$label" "$triple" "$tool" "$features"
    done
    _darwin_rows
}

# Every target that belongs in a full release, regardless of host — used by
# release.sh to check completeness. Commented rows are excluded; macOS is always
# expected. Emits "label|triple|tool|features".
all_release_targets() {
    _matrix_rows | while IFS='|' read -r label triple tool features want; do
        printf '%s|%s|%s|%s\n' "$label" "$triple" "$tool" "$features"
    done
    echo "macOS ARM|aarch64-apple-darwin|native|clipboard"
    echo "macOS Intel|x86_64-apple-darwin|native|clipboard"
}

# Triples whose binary is already staged for the current release id, one per
# line. The id pins version + commit, so anything found here was built from
# exactly the current source and needn't be rebuilt. Silent and empty when
# staging isn't configured or the server can't be reached.
staged_triples() {
    staging_configured || return 0
    local dest
    dest="$BACON_STAGE_DIR/$(release_id)"
    ssh "$BACON_STAGE_HOST" \
        "ls -1 '$dest'/*/'$NAME' '$dest'/*/'$NAME.exe' 2>/dev/null" 2>/dev/null \
        | sed -n 's:.*/\([^/]*\)/[^/]*$:\1:p' | sort -u || true
}

# The binary path inside build/ for a triple: build/<triple>/bacon[.exe]
target_binary() { # target_binary <triple>
    local triple=$1 exe=$NAME
    [[ $triple == *windows* ]] && exe="$NAME.exe"
    printf 'build/%s/%s\n' "$triple" "$exe"
}

# What `file` must report for a triple's binary. A binary served under the wrong
# triple would be silently unusable in someone's CI, so the arch is checked
# rather than trusted.
_arch_pattern() { # _arch_pattern <triple>
    case ${1%%.*} in
        x86_64-unknown-linux-*)  echo 'ELF 64-bit.*(x86-64|x86_64)' ;;
        aarch64-unknown-linux-*) echo 'ELF 64-bit.*(aarch64|ARM aarch64)' ;;
        x86_64-pc-windows-*)     echo '(PE32\+|MS Windows).*(x86-64|x86_64)' ;;
        aarch64-apple-darwin)    echo 'Mach-O 64-bit.*arm64' ;;
        x86_64-apple-darwin)     echo 'Mach-O 64-bit.*x86_64' ;;
        *) echo '' ;;
    esac
}

# Check a freshly built binary exists and is what it claims to be: right arch,
# static for musl, no duplicate linked dylib on macOS, and — when it's for this
# very host — that it actually runs and reports the expected version.
verify_binary() { # verify_binary <path> <triple>
    local bin=$1 triple=$2 desc pattern
    [[ -f $bin ]] || die "expected binary not produced: $bin"
    if have file; then
        desc=$(file -b "$bin")
        info "$desc"
        pattern=$(_arch_pattern "$triple")
        if [[ -n $pattern ]]; then
            [[ $desc =~ $pattern ]] || die "$bin doesn't look like a $triple binary"
        fi
        # A musl binary that isn't static defeats the point of shipping it.
        if [[ $triple == *-musl* && $desc == *"dynamically linked"* ]]; then
            warn "$triple binary is dynamically linked"
        fi
    fi
    if [[ $triple == *-apple-darwin ]] && have otool; then
        local dups
        dups=$(otool -L "$bin" | sed -n 's/^[[:space:]]\{1,\}\([^ ]*\).*/\1/p' | sort | uniq -d)
        [[ -z $dups ]] || die "duplicate linked dylib(s) in $bin:"$'\n'"$dups"
        ok "no duplicate dylibs"
    fi
    if [[ ${triple%%.*} == "$(host_target)" ]]; then
        local reported expected
        expected=$(bacon_version)
        reported=$("$bin" --version 2>/dev/null | tr -dc '0-9.' || true)
        [[ $reported == "$expected" ]] || die "$bin reports version '$reported', expected '$expected'"
        ok "runs here, reports $expected"
    fi
}

# Build one target described by a "label|triple|tool|features" row and copy the
# resulting binary into build/<triple>/.
build_row() { # build_row "<label>|<triple>|<tool>|<features>"
    local label triple tool features
    IFS='|' read -r label triple tool features <<< "$1"

    local tdir="$CACHE/$triple" exe=$NAME bin feat=()
    [[ -n $features ]] && feat=(--features "$features")
    [[ $triple == *windows* ]] && exe="$NAME.exe"

    h2 "$label   (target=$triple, tool=$tool, features='${features:-none}')"
    case $tool in
        native)
            need cargo "install the Rust toolchain — https://rustup.rs"
            rustup target add "$triple" >/dev/null   # idempotent
            cargo build --release --locked --target "$triple" \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/$triple/release/$exe" ;;
        zig)
            need cargo-zigbuild "cargo install cargo-zigbuild  (and install zig)"
            need zig "brew install zig  (or see https://ziglang.org)"
            rustup target add "${triple%%.*}" >/dev/null   # ensure rust-std for the target (idempotent)
            cargo zigbuild --release --locked --target "$triple" \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/${triple%%.*}/release/$exe" ;; # zigbuild drops any glibc suffix from the dir name
        *) die "unknown build tool '$tool' for target $label" ;;
    esac

    verify_binary "$bin" "$triple"
    mkdir -p "build/$triple"
    cp "$bin" "build/$triple/"
    ok "$label → build/$triple/$exe"
}
