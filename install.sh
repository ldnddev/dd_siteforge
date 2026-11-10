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
#   ./install.sh uninstall        # remove the binary, theme, launcher, and icon
#   ./install.sh --help           # this help
#
# On Linux, install writes a desktop launcher so the TUI shows in the
# application menu (Terminal=true). A prefix under $HOME publishes it in
# ${XDG_DATA_HOME:-$HOME/.local/share}/applications. Any other prefix uses
# $PREFIX/share/applications. The icon is copied when this package includes
# source/favicon/web-app-manifest-512x512.png.
#
# Pin a version or override locations:
#   VERSION=v1.31.1                # default: latest GitHub Release
#   PREFIX=$HOME/.local           # binary lives at $PREFIX/bin/dd_siteforge
#   CONFIG_DIR=$HOME/.config/ldnddev
#   XDG_DATA_HOME=...             # launcher and icon live under here when set
#   TARGET=x86_64-unknown-linux-musl   # skip auto-detect
#
# Piped extras:
#   curl -fsSL … | bash -s -- --version v1.31.1
#   curl -fsSL … | bash -s -- uninstall
#
# Re-run safe: existing themes are left alone on install; the binary and
# launcher are overwritten. Uninstall removes the binary, the theme file,
# the desktop launcher, and the installed icon. It leaves
# $CONFIG_DIR/dd_siteforge/ (session and scaffold) in place.
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
  ./install.sh uninstall        remove the binary, theme, launcher, and icon
  ./install.sh --help           this help

On Linux the install also writes a desktop launcher (Terminal=true) so the
TUI shows in the application menu. Uninstall removes that launcher and the
installed icon. Session and scaffold files under the config dir stay.
Pass the same PREFIX, BIN_DIR, CONFIG_DIR, and XDG_DATA_HOME you used
to install.

Override defaults via env vars:
  VERSION=v1.31.1                pin a release (default: latest)
  PREFIX=$HOME/.local           binary lives at $PREFIX/bin/dd_siteforge
  BIN_DIR, CONFIG_DIR, XDG_DATA_HOME, TARGET, GITHUB_REPO

Piped extras:
  curl -fsSL … | bash -s -- --version v1.31.1
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
    # Empty → latest. Accept "1.31.1" or "v1.31.1".
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
    [ -s "$sumfile" ] || { red "Checksum file missing or empty — refusing to install an unverified binary."; exit 1; }

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
        red "No sha256sum or shasum on PATH — cannot verify $base."
        exit 1
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

# Desktop menus search XDG_DATA_HOME, then the system data dirs.
# A prefix under $HOME still publishes the launcher where the session looks.
data_dir() {
    if [ -n "${XDG_DATA_HOME:-}" ]; then
        printf '%s\n' "$XDG_DATA_HOME"
        return 0
    fi
    case "$PREFIX" in
        "$HOME"|"$HOME"/*)
            printf '%s/.local/share\n' "$HOME"
            ;;
        *)
            printf '%s/share\n' "$PREFIX"
            ;;
    esac
}

apps_dir() {
    printf '%s/applications\n' "$(data_dir)"
}

icon_path() {
    printf '%s/icons/hicolor/512x512/apps/%s.png\n' "$(data_dir)" "$BIN_NAME"
}

# Quote one Exec argument. A literal % is %%.
desktop_quote() {
    local s=$1
    case "$s" in
        *$'\n'*) red "Desktop path contains a newline"; exit 1 ;;
    esac
    s=${s//\\/\\\\}
    s=${s//\"/\\\"}
    s=${s//\`/\\\`}
    s=${s//\$/\\\$}
    s=${s//%/%%}
    printf '"%s"' "$s"
}

is_png() {
    local magic
    magic="$(od -An -t x1 -N 8 -- "$1" 2>/dev/null | tr -d ' \n')"
    [ "$magic" = "89504e470d0a1a0a" ]
}

# Copy the brand mark when this package has one. Returns 0 when the icon
# is installed at icon_path, 1 when the package has no PNG.
install_icon() {
    local search_dir="${1:-}" src=""
    local dest
    dest="$(icon_path)"

    if in_repo; then
        local local_icon
        local_icon="$(repo_root)/source/favicon/web-app-manifest-512x512.png"
        if [ -f "$local_icon" ]; then
            src="$local_icon"
        fi
    fi
    if [ -z "$src" ] && [ -n "$search_dir" ] && [ -d "$search_dir" ]; then
        src="$(find "$search_dir" -type f -name 'web-app-manifest-512x512.png' 2>/dev/null | head -n 1 || true)"
    fi
    if [ -z "$src" ]; then
        yellow "No icon in this package — the launcher uses the menu's default icon."
        return 1
    fi
    if ! is_png "$src"; then
        yellow "Icon at $src is not a PNG — the launcher uses the menu's default icon."
        return 1
    fi

    mkdir -p "$(dirname "$dest")"
    install_file 0644 "$src" "$dest"
    green "Installed icon → $dest"
    return 0
}

write_desktop() {
    local dest=$1 bin=$2 icon=${3:-}
    local exec_q
    exec_q="$(desktop_quote "$bin")"
    {
        printf '%s\n' "[Desktop Entry]"
        printf '%s\n' "Type=Application"
        printf '%s\n' "Version=1.0"
        printf '%s\n' "Name=${BIN_NAME}"
        printf '%s\n' "GenericName=Site editor"
        printf '%s\n' "Comment=Edit a static site in the terminal"
        printf '%s\n' "Exec=${exec_q}"
        printf '%s\n' "TryExec=${bin}"
        printf '%s\n' "Terminal=true"
        printf '%s\n' "Categories=Development;"
        printf '%s\n' "Keywords=site;cms;html;static;tui;"
        printf '%s\n' "StartupNotify=false"
        if [ -n "$icon" ]; then
            printf '%s\n' "Icon=${icon}"
        fi
    } >"$dest"
    chmod 644 "$dest"
}

install_launcher() {
    local search_dir="${1:-}"
    case "$(uname -s)" in
        Linux) ;;
        *)
            yellow "Desktop launcher is written on Linux. Skipped on $(uname -s)."
            return 0
            ;;
    esac

    local dir dest bin icon=""
    dir="$(apps_dir)"
    dest="$dir/${BIN_NAME}.desktop"
    bin="$BIN_DIR/$BIN_NAME"
    mkdir -p "$dir"
    if install_icon "$search_dir"; then
        icon="$(icon_path)"
    fi
    write_desktop "$dest" "$bin" "$icon"
    green "Launcher $dest"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$dir" >/dev/null 2>&1 || true
    fi
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
    install_launcher "${1:-}"
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

    if ! http_get "$sum_url" "$tmp/$asset.sha256" 2>/dev/null; then
        red "No checksum published for this asset — refusing to install an unverified binary."
        red "  $sum_url"
        exit 1
    fi
    verify_sha256 "$archive" "$tmp/$asset.sha256"

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
    finish_ok "$tmp"
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
    local desktop_dst icon_dst
    desktop_dst="$(apps_dir)/${BIN_NAME}.desktop"
    icon_dst="$(icon_path)"
    local removed_any=0
    local path

    for path in "$bin_dst" "$theme_dst" "$desktop_dst" "$icon_dst"; do
        if [ -f "$path" ] || [ -L "$path" ]; then
            rm -f -- "$path"
            green "Removed $path"
            removed_any=1
        else
            yellow "Not installed: $path"
        fi
    done

    # Remove config dir only if empty (don't clobber other tools' files)
    if [ -d "$CONFIG_DIR" ] && [ -z "$(ls -A "$CONFIG_DIR" 2>/dev/null)" ]; then
        rmdir "$CONFIG_DIR"
        green "Removed empty $CONFIG_DIR"
    fi

    local apps session_dir
    apps="$(apps_dir)"
    if command -v update-desktop-database >/dev/null 2>&1 && [ -d "$apps" ]; then
        update-desktop-database "$apps" >/dev/null 2>&1 || true
    fi

    session_dir="$CONFIG_DIR/$BIN_NAME"
    if [ -d "$session_dir" ]; then
        yellow "Left $session_dir in place (session and scaffold, not written by this installer)."
    fi

    if [ "$removed_any" -eq 0 ]; then
        yellow "Nothing to uninstall."
    else
        green ""
        green "Uninstall complete."
    fi
}

# ---- dispatch ---------------------------------------------------------------
main() {
cmd="install"
mode="auto"

while [ $# -gt 0 ]; do
    case "$1" in
        install)            cmd="install"; shift ;;
        uninstall|remove|--uninstall)
                            cmd="uninstall"; shift ;;
        --from-source)      mode="source"; shift ;;
        --from-release|--binary)
                            mode="release"; shift ;;
        --version)
            [ $# -ge 2 ] || { red "--version needs a value (e.g. v1.31.1)"; exit 1; }
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
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    main "$@"
fi
