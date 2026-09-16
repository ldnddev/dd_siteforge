#!/usr/bin/env bash
# install.sh — install dd_siteforge from a GitHub Release, or build from source.
#
# One-liner (picks the binary for this machine):
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash
#
# Usage:
#   ./install.sh                  # from a clone: cargo build --release and install
#                                 # otherwise: download the matching GitHub Release
#   ./install.sh install          # same as above
#   ./install.sh --from-release   # always download the matching GitHub Release
#   ./install.sh --from-source    # always cargo-build (clone only)
#   ./install.sh uninstall        # remove the binary + theme + config dir if empty
#   ./install.sh --help           # this help
#
# Pin a version or override locations:
#   VERSION=v1.9.0                # default: latest GitHub Release
#   PREFIX=$HOME/.local           # binary lives at $PREFIX/bin/dd_siteforge
#   CONFIG_DIR=$HOME/.config/ldnddev
#   TARGET=x86_64-unknown-linux-musl   # skip auto-detect
#
# Piped extras:
#   curl -fsSL … | bash -s -- --version v1.9.0
#   curl -fsSL … | bash -s -- uninstall
#
# Re-run safe: existing themes are left alone on install; the binary is overwritten.
# The Grunt/source/Lando/DDEV kit is embedded in the binary and seeded by
# `dd_siteforge init-site`. Optional house overlay: `dd_siteforge init-scaffold --global`.

set -euo pipefail

# ---- config -----------------------------------------------------------------
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev}"
BIN_NAME="dd_siteforge"
THEME_FILE="dd_siteforge_theme.yml"
GITHUB_REPO="${GITHUB_REPO:-ldnddev/dd_siteforge}"
VERSION="${VERSION:-}"
TARGET="${TARGET:-}"

# ---- pretty -----------------------------------------------------------------
if [ -t 1 ]; then
    cyan()   { printf '\033[36m%s\033[0m\n' "$*"; }
    green()  { printf '\033[32m%s\033[0m\n' "$*"; }
    yellow() { printf '\033[33m%s\033[0m\n' "$*"; }
    red()    { printf '\033[31m%s\033[0m\n' "$*" >&2; }
else
    cyan()   { printf '%s\n' "$*"; }
    green()  { printf '%s\n' "$*"; }
    yellow() { printf '%s\n' "$*"; }
    red()    { printf '%s\n' "$*" >&2; }
fi

require() {
    command -v "$1" >/dev/null 2>&1 || { red "Required command not found: $1"; exit 1; }
}

usage() {
    cat <<'EOF'
install.sh — install dd_siteforge from a GitHub Release, or build from source.

One-liner (picks the binary for this machine):
  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash

Usage:
  ./install.sh                  from a clone: cargo build --release and install
                                otherwise: download the matching GitHub Release
  ./install.sh install          same as above
  ./install.sh --from-release   always download the matching GitHub Release
  ./install.sh --from-source    always cargo-build (clone only)
  ./install.sh uninstall        remove the binary + theme + empty config dir
  ./install.sh --help           this help

Override defaults via env vars:
  VERSION=v1.9.0                pin a release (default: latest)
  PREFIX=$HOME/.local           binary lives at $PREFIX/bin/dd_siteforge
  BIN_DIR, CONFIG_DIR, TARGET, GITHUB_REPO

Piped extras:
  curl -fsSL … | bash -s -- --version v1.9.0
  curl -fsSL … | bash -s -- uninstall
EOF
}

# True when this script is sitting next to Cargo.toml (a git clone / tarball).
# False when piped via `curl | bash` (BASH_SOURCE is empty, bash, or /dev/fd/N).
in_repo() {
    local src dir
    src="${BASH_SOURCE[0]:-}"
    case "$src" in
        ''|bash|sh|-|/dev/fd/*|/proc/self/fd/*) return 1 ;;
    esac
    [ -f "$src" ] || return 1
    dir="$(cd "$(dirname "$src")" && pwd)" || return 1
    [ -f "$dir/Cargo.toml" ] && [ -f "$dir/$THEME_FILE" ]
}

repo_root() {
    cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

# ---- platform ---------------------------------------------------------------
detect_target() {
    if [ -n "$TARGET" ]; then
        return 0
    fi

    local kernel machine
    kernel="$(uname -s | tr '[:upper:]' '[:lower:]')"
    machine="$(uname -m | tr '[:upper:]' '[:lower:]')"

    case "$machine" in
        x86_64|amd64)  machine="x86_64" ;;
        aarch64|arm64) machine="aarch64" ;;
        *)
            red "Unsupported CPU architecture: $machine"
            red "Supported: x86_64, aarch64. Build from source with cargo if you need another target."
            exit 1
            ;;
    esac

    case "$kernel" in
        linux)  TARGET="${machine}-unknown-linux-musl" ;;
        darwin) TARGET="${machine}-apple-darwin" ;;
        mingw*|msys*|cygwin*)
            red "This installer is for Linux and macOS."
            red "On Windows, download a release zip from https://github.com/${GITHUB_REPO}/releases"
            red "or: cargo install --git https://github.com/${GITHUB_REPO} --locked"
            exit 1
            ;;
        *)
            red "Unsupported OS: $kernel"
            red "Supported: Linux, macOS. Or build from source with cargo."
            exit 1
            ;;
    esac
}

normalize_version() {
    # Empty → latest. Accept "1.9.0" or "v1.9.0".
    if [ -z "$VERSION" ] || [ "$VERSION" = "latest" ]; then
        VERSION=""
        return 0
    fi
    case "$VERSION" in
        v*) ;;
        *) VERSION="v$VERSION" ;;
    esac
}

asset_base() {
    printf '%s-%s' "$BIN_NAME" "$TARGET"
}

release_url() {
    local asset="$1"
    if [ -n "$VERSION" ]; then
        printf 'https://github.com/%s/releases/download/%s/%s' "$GITHUB_REPO" "$VERSION" "$asset"
    else
        printf 'https://github.com/%s/releases/latest/download/%s' "$GITHUB_REPO" "$asset"
    fi
}

# ---- download ---------------------------------------------------------------
http_get() {
    local url="$1" dest="$2"
    if command -v curl >/dev/null 2>&1; then
        set -- curl -fL --retry 3 --retry-delay 1 --connect-timeout 15
        if [ -n "${GITHUB_TOKEN:-}" ]; then
            set -- "$@" -H "Authorization: Bearer $GITHUB_TOKEN" -H "Accept: application/octet-stream"
        fi
        if [ -t 2 ]; then
            set -- "$@" --progress-bar
        else
            set -- "$@" -sS
        fi
        "$@" -o "$dest" "$url" || return $?
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$dest" "$url" || return $?
    else
        red "Need curl or wget to download a release."
        return 1
    fi
}

verify_sha256() {
    local archive="$1" sumfile="$2"
    local expected got dir base
    [ -s "$sumfile" ] || { yellow "Checksum file missing or empty — skipping verify."; return 0; }

    dir="$(dirname "$archive")"
    base="$(basename "$archive")"

    if grep -q '[[:space:]]' "$sumfile" 2>/dev/null; then
        if command -v sha256sum >/dev/null 2>&1; then
            (cd "$dir" && sha256sum -c "$sumfile")
            return 0
        fi
        if command -v shasum >/dev/null 2>&1; then
            (cd "$dir" && shasum -a 256 -c "$sumfile")
            return 0
        fi
    fi

    expected="$(awk '{print $1; exit}' "$sumfile" | tr -d '[:space:]')"
    if command -v sha256sum >/dev/null 2>&1; then
        got="$(sha256sum "$archive" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        got="$(shasum -a 256 "$archive" | awk '{print $1}')"
    else
        yellow "No sha256sum/shasum on PATH — skipping checksum for $base."
        return 0
    fi

    if [ "$expected" != "$got" ]; then
        red "Checksum mismatch for $base"
        red "  expected: $expected"
        red "  got:      $got"
        exit 1
    fi
    green "Checksum OK ($base)"
}

install_file() {
    local mode="$1" src="$2" dst="$3"
    if command -v install >/dev/null 2>&1; then
        install -m "$mode" "$src" "$dst"
    else
        cp "$src" "$dst"
        chmod "$mode" "$dst"
    fi
}

warn_path() {
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            yellow ""
            yellow "Note: $BIN_DIR is not on \$PATH. Add it to your shell rc:"
            yellow "    export PATH=\"$BIN_DIR:\$PATH\""
            ;;
    esac
}

install_theme_if_absent() {
    local src="$1"
    mkdir -p "$CONFIG_DIR"
    local dst="$CONFIG_DIR/$THEME_FILE"
    if [ -f "$dst" ]; then
        yellow "Theme already exists at $dst — leaving it alone."
        return 0
    fi
    if [ ! -f "$src" ]; then
        yellow "No $THEME_FILE in this package — skipping theme (built-in defaults still apply)."
        return 0
    fi
    install_file 0644 "$src" "$dst"
    green "Installed default theme → $dst"
}

finish_ok() {
    warn_path
    green ""
    green "Done. Try:  $BIN_NAME --help"
}

# ---- subcommands ------------------------------------------------------------
do_install_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        red "Required command not found: cargo"
        yellow "Hint: download a prebuilt binary with  ./install.sh --from-release"
        yellow "  or: curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash"
        exit 1
    fi

    local root src_bin
    root="$(repo_root)"
    cd "$root"

    [ -f Cargo.toml ] || { red "No Cargo.toml in $root — run install.sh from the repo root, or omit --from-source."; exit 1; }
    [ -f "$THEME_FILE" ] || { red "Missing $THEME_FILE in $root."; exit 1; }

    cyan "Building $BIN_NAME (release)…"
    cargo build --release

    src_bin="target/release/$BIN_NAME"
    [ -x "$src_bin" ] || { red "Build did not produce $src_bin"; exit 1; }

    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$BIN_NAME"
    green "Installed $BIN_NAME → $BIN_DIR/$BIN_NAME"

    install_theme_if_absent "$THEME_FILE"
    finish_ok
}

do_install_from_release() {
    detect_target
    normalize_version

    local asset archive_url sum_url tmp archive src_bin theme_src
    asset="$(asset_base).tar.gz"
    archive_url="$(release_url "$asset")"
    sum_url="$(release_url "$asset.sha256")"

    cyan "Detected target: $TARGET"
    if [ -n "$VERSION" ]; then
        cyan "Downloading $BIN_NAME $VERSION ($asset)…"
    else
        cyan "Downloading $BIN_NAME (latest $asset)…"
    fi

    tmp=""
    cleanup() { [ -n "${tmp:-}" ] && rm -rf "$tmp"; }
    trap cleanup EXIT
    tmp="$(mktemp -d)"
    archive="$tmp/$asset"

    if ! http_get "$archive_url" "$archive"; then
        red "Failed to download:"
        red "  $archive_url"
        red ""
        red "No prebuilt package for $TARGET (or that release has no assets yet)."
        red "Options:"
        red "  • From a clone:  ./install.sh --from-source"
        red "  • With cargo:    cargo install --git https://github.com/${GITHUB_REPO} --locked"
        red "  • Releases:      https://github.com/${GITHUB_REPO}/releases"
        exit 1
    fi

    if http_get "$sum_url" "$tmp/$asset.sha256" 2>/dev/null; then
        verify_sha256 "$archive" "$tmp/$asset.sha256"
    else
        yellow "No checksum published for this asset — skipping verify."
    fi

    tar -xzf "$archive" -C "$tmp"

    src_bin="$tmp/$BIN_NAME"
    if [ ! -f "$src_bin" ]; then
        src_bin="$(find "$tmp" -type f -name "$BIN_NAME" | head -n 1 || true)"
    fi
    [ -n "$src_bin" ] && [ -f "$src_bin" ] || { red "Archive did not contain $BIN_NAME"; exit 1; }
    chmod +x "$src_bin"

    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$BIN_NAME"
    green "Installed $BIN_NAME → $BIN_DIR/$BIN_NAME"

    theme_src="$tmp/$THEME_FILE"
    if [ ! -f "$theme_src" ]; then
        theme_src="$(find "$tmp" -type f -name "$THEME_FILE" | head -n 1 || true)"
    fi
    install_theme_if_absent "${theme_src:-}"
    finish_ok
}

do_install() {
    local mode="$1"
    case "$mode" in
        source)  do_install_from_source ;;
        release) do_install_from_release ;;
        auto)
            if in_repo; then
                if ! command -v cargo >/dev/null 2>&1; then
                    yellow "No cargo on PATH — downloading a prebuilt binary instead."
                    yellow "(Use --from-source once you have a Rust toolchain.)"
                    do_install_from_release
                else
                    do_install_from_source
                fi
            else
                do_install_from_release
            fi
            ;;
        *) red "internal: unknown install mode $mode"; exit 1 ;;
    esac
}

do_uninstall() {
    local bin_dst="$BIN_DIR/$BIN_NAME"
    local theme_dst="$CONFIG_DIR/$THEME_FILE"
    local removed_any=0

    if [ -f "$bin_dst" ] || [ -L "$bin_dst" ]; then
        rm -f "$bin_dst"
        green "Removed $bin_dst"
        removed_any=1
    else
        yellow "No binary at $bin_dst"
    fi

    if [ -f "$theme_dst" ] || [ -L "$theme_dst" ]; then
        rm -f "$theme_dst"
        green "Removed $theme_dst"
        removed_any=1
    else
        yellow "No theme at $theme_dst"
    fi

    # Remove config dir only if empty (don't clobber other tools' files)
    if [ -d "$CONFIG_DIR" ] && [ -z "$(ls -A "$CONFIG_DIR")" ]; then
        rmdir "$CONFIG_DIR"
        green "Removed empty $CONFIG_DIR"
    fi

    if [ "$removed_any" -eq 0 ]; then
        yellow "Nothing to uninstall."
    else
        green ""
        green "Uninstall complete."
    fi
}

# ---- dispatch ---------------------------------------------------------------
cmd="install"
mode="auto"

while [ $# -gt 0 ]; do
    case "$1" in
        install)            cmd="install"; shift ;;
        uninstall|remove)   cmd="uninstall"; shift ;;
        --from-source)      mode="source"; shift ;;
        --from-release|--binary)
                            mode="release"; shift ;;
        --version)
            [ $# -ge 2 ] || { red "--version needs a value (e.g. v1.9.0)"; exit 1; }
            VERSION="$2"
            shift 2
            ;;
        --version=*)        VERSION="${1#*=}"; shift ;;
        --target)
            [ $# -ge 2 ] || { red "--target needs a rust triple"; exit 1; }
            TARGET="$2"
            shift 2
            ;;
        --target=*)         TARGET="${1#*=}"; shift ;;
        -h|--help|help)     usage; exit 0 ;;
        *)
            red "Unknown argument: $1"
            usage
            exit 1
            ;;
    esac
done

case "$cmd" in
    install)
        if [ "$mode" = "source" ] && ! in_repo; then
            red "--from-source needs a clone of the repo (Cargo.toml next to install.sh)."
            red "To install a prebuilt binary:"
            red "  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash"
            exit 1
        fi
        do_install "$mode"
        ;;
    uninstall) do_uninstall ;;
    *) red "Unknown command: $cmd"; usage; exit 1 ;;
esac
