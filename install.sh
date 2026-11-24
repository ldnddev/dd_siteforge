#!/usr/bin/env bash
# install.sh — install dd_siteforge from a GitHub Release, or build from source.
#
# From a clone:
#   ./install.sh                 # ask: terminal, window, or both
#   ./install.sh --tui
#   ./install.sh --gui
#   ./install.sh --both
#   ./install.sh --uninstall     # ask the same question
#
# One-liner (curl):
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash -s -- --both
#   curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash -s -- --uninstall
#
# Env:
#   PREFIX       install prefix (default: ~/.local)
#   BIN_DIR      override binary dir (default: $PREFIX/bin)
#   CONFIG_DIR   theme dir (default: ~/.config/ldnddev)
#   VERSION      release tag (default: latest GitHub Release)
#   TARGET       rust triple for a single program (skip auto-detect)
#   GITHUB_REPO  default ldnddev/dd_siteforge
#   INSTALL_GUI  set to 1 for the same result as --both
#
# On Linux, install writes a desktop launcher for each binary it installs.
# --tui is the musl terminal app. --gui is the glibc window. --both installs
# both. A release install checks the published SHA-256 and refuses a mismatch.
# --uninstall asks which programs to remove, then leaves the theme, the icon,
# and $CONFIG_DIR/dd_siteforge (session and scaffold) in place. With no
# terminal to answer, install selects the terminal app and uninstall removes
# both. Pass the same PREFIX, BIN_DIR, and XDG_DATA_HOME you used to install.

set -euo pipefail

# ---- config -----------------------------------------------------------------
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/ldnddev}"
TUI_BIN="dd_siteforge"
GUI_BIN="dd_siteforge-gui"
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

# ldnddev wordmark, half-block art of the white-on-black mark.
# Printed only on a terminal so a captured log stays plain text.
print_mark() {
    [ -t 1 ] || return 0
    printf '\033[40;97m'
    cat <<'EOF'

█▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█
█  ██                ██                         ██  █
█  ██                ██                         ██  █
█  ██         ▄▄▄▄▄▄ ██     ▄▄▄▄▄▄       ▄▄▄▄▄▄ ██  █
█  ██       ▄██▀▀▀▀▀███  ▄██████████▄  ▄██▀▀▀▀▀███  █
█  ██      ██▀       ██  ██        ██ ██▀       ██  █
█  ██      ██        ██  ██        ██ ██        ██  █
█  ██      ▀█▄       ██  ██        ██ ▀█▄       ██  █
█  ▀█▄▄▄▄▄  ▀██▄▄▄▄██▀   ██        ██  ▀██▄▄▄▄██▀   █
█   ▀▀▀▀▀▀    ▀▀▀▀▀▀     ▀▀        ▀▀    ▀▀▀▀▀▀     █
█                                                   █
█                     ██                            █
█                     ██                            █
█              ▄▄▄▄▄▄ ██     ▄▄▄▄▄▄    ▄▄      ▄▄▄  █
█            ▄██▀▀▀▀▀███   ██▀▀▀▀███   ██     ▄██   █
█           ██▀       ██  ██    ██▀ ▄▄ ██    ▄██    █
█           ██        ██  ██  ▄██   ██ ██    ██     █
█           ▀█▄       ██  ██▄██▀   ▄█▀ ██  ▄██      █
█  ▄▄▄▄▄▄▄▄  ▀██▄▄▄▄██▀    ▀███▄▄▄██▀  ██▄██▀       █
█  ▀▀▀▀▀▀▀▀    ▀▀▀▀▀▀        ▀▀▀▀▀▀    ▀▀▀▀         █
▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
EOF
    printf '\033[0m\n'
}

require() {
    command -v "$1" >/dev/null 2>&1 || { red "Required command not found: $1"; exit 1; }
}

usage() {
    cat <<'EOF'
Usage: install.sh [--tui | --gui | --both] [--from-source | --from-release] [--uninstall]

  ./install.sh
      ask: terminal app, desktop window, or both
      from a clone this builds; otherwise it downloads the GitHub Release
  ./install.sh --tui
      install dd_siteforge
  ./install.sh --gui
      install dd_siteforge-gui
  ./install.sh --both
      install both
  ./install.sh --from-source
      always cargo-build (a clone only)
  ./install.sh --from-release
      always download the matching GitHub Release
  ./install.sh --uninstall
      ask which of those to remove
  ./install.sh --uninstall --tui
  ./install.sh --uninstall --gui
  ./install.sh --uninstall --both

With no terminal to answer, install selects the terminal app and uninstall
removes both. INSTALL_GUI=1 is the same as --both.

Linux TUI releases are musl. Linux GUI releases are glibc. Tags before
v1.32.0 have no GUI asset, so install that window from a clone.
Uninstall removes the chosen binaries and their launchers. The theme, the
icon, and the session directory stay.

Env: PREFIX, BIN_DIR, CONFIG_DIR, XDG_DATA_HOME, VERSION, TARGET, GITHUB_REPO,
     INSTALL_GUI=1
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
# MACHINE is the rust arch (x86_64 or aarch64).
detect_machine() {
    if [ -n "${MACHINE:-}" ]; then
        return 0
    fi
    local machine kernel
    machine="$(uname -m | tr '[:upper:]' '[:lower:]')"
    kernel="$(uname -s | tr '[:upper:]' '[:lower:]')"
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
        linux|darwin) ;;
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
    MACHINE="$machine"
    KERNEL="$kernel"
}

# Print the rust triple for tui or gui. A set TARGET applies to one program.
# --both always uses musl for the terminal app and glibc for the window.
resolve_triple() {
    local kind=$1
    detect_machine
    if [ -n "$TARGET" ] && [ "${INSTALL_TARGET:-}" != both ]; then
        printf '%s\n' "$TARGET"
        return 0
    fi
    case "$KERNEL" in
        linux)
            if [ "$kind" = gui ]; then
                printf '%s-unknown-linux-gnu\n' "$MACHINE"
            else
                printf '%s-unknown-linux-musl\n' "$MACHINE"
            fi
            ;;
        darwin)
            printf '%s-apple-darwin\n' "$MACHINE"
            ;;
        *)
            red "Unsupported OS: $KERNEL"
            exit 1
            ;;
    esac
}

normalize_version() {
    # Empty → latest. Accept "1.32.0" or "v1.32.0".
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
    printf '%s-%s' "$1" "$2"
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
    printf '%s/icons/hicolor/512x512/apps/dd_siteforge.png\n' "$(data_dir)"
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
    local dest=$1 name=$2 comment=$3 bin=$4 terminal=$5 wmclass=${6:-} icon=${7:-}
    local exec_q
    exec_q="$(desktop_quote "$bin")"
    {
        printf '%s\n' "[Desktop Entry]"
        printf '%s\n' "Type=Application"
        printf '%s\n' "Version=1.0"
        printf '%s\n' "Name=${name}"
        printf '%s\n' "GenericName=Site editor"
        printf '%s\n' "Comment=${comment}"
        printf '%s\n' "Exec=${exec_q}"
        printf '%s\n' "TryExec=${bin}"
        printf '%s\n' "Terminal=${terminal}"
        printf '%s\n' "Categories=Development;"
        printf '%s\n' "Keywords=site;cms;html;static;"
        printf '%s\n' "StartupNotify=false"
        if [ -n "$wmclass" ]; then
            printf '%s\n' "StartupWMClass=${wmclass}"
        fi
        if [ -n "$icon" ]; then
            printf '%s\n' "Icon=${icon}"
        fi
    } >"$dest"
    chmod 644 "$dest"
}

install_launchers() {
    local search_dir="${1:-}"
    case "$(uname -s)" in
        Linux) ;;
        *)
            yellow "Desktop launchers are written on Linux. Skipped on $(uname -s)."
            return 0
            ;;
    esac

    local dir icon=""
    dir="$(apps_dir)"
    mkdir -p "$dir"
    if install_icon "$search_dir"; then
        icon="$(icon_path)"
    fi
    if [ -x "$BIN_DIR/$TUI_BIN" ]; then
        write_desktop \
            "$dir/${TUI_BIN}.desktop" \
            "dd_siteforge" \
            "Edit a static site in the terminal" \
            "$BIN_DIR/$TUI_BIN" \
            "true" \
            "" \
            "$icon"
        green "Launcher $dir/${TUI_BIN}.desktop"
    fi
    if [ -x "$BIN_DIR/$GUI_BIN" ]; then
        write_desktop \
            "$dir/${GUI_BIN}.desktop" \
            "dd_siteforge GUI" \
            "Edit a static site in a window" \
            "$BIN_DIR/$GUI_BIN" \
            "false" \
            "dev.ldnddev.dd_siteforge" \
            "$icon"
        green "Launcher $dir/${GUI_BIN}.desktop"
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$dir" >/dev/null 2>&1 || true
    fi
}

# Read 1, 2, or 3 from the terminal. stdout is tui, gui, or both.
# $1 is the verb shown in the question. $2 is the default number.
# Returns 1 when the terminal cannot be used. main captures stdout, and a
# failed redirect in that capture does not abort under set -e, so every
# /dev/tty failure has to be an explicit status.
ask_choice() {
    local verb=$1 default=$2 answer choice
    while true; do
        {
            printf '%s which program?\n' "$verb"
            printf '%s\n' "  1) terminal (dd_siteforge)"
            printf '%s\n' "  2) desktop window (dd_siteforge-gui)"
            printf '%s\n' "  3) both"
            printf 'Choice [%s]: ' "$default"
        } >/dev/tty 2>/dev/null || return 1
        if ! IFS= read -r answer </dev/tty 2>/dev/null; then
            printf '\n' >/dev/tty 2>/dev/null || true
            answer=$default
        fi
        choice=$(printf '%s' "$answer" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')
        case "$choice" in
            '' ) choice=$default ;;
        esac
        case "$choice" in
            1 | tui | terminal) printf '%s\n' tui; return 0 ;;
            2 | gui | window) printf '%s\n' gui; return 0 ;;
            3 | both) printf '%s\n' both; return 0 ;;
            *) printf '%s\n' "Enter 1, 2, or 3." >/dev/tty 2>/dev/null || return 1 ;;
        esac
    done
}

# The device node is mode rw-rw-rw, so -r/-w stay true with no controlling
# terminal. Opening it is the check that matters.
have_tty() {
    { : <>/dev/tty; } 2>/dev/null
}

choose_target() {
    local action=$1 explicit=$2
    if [ -n "$explicit" ]; then
        printf '%s\n' "$explicit"
        return 0
    fi
    if have_tty; then
        if [ "$action" = uninstall ]; then
            ask_choice "Remove" 3 && return 0
        else
            ask_choice "Install" 1 && return 0
        fi
    fi
    if [ "$action" = uninstall ]; then
        printf '%s\n' "No terminal for a prompt. Removing both programs." >&2
        printf '%s\n' both
    else
        printf '%s\n' "No terminal for a prompt. Installing the terminal app." >&2
        printf '%s\n' tui
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

finish_note() {
    warn_path
    green ""
    case "$INSTALL_TARGET" in
        tui) green "Done. Try:  $TUI_BIN --help" ;;
        gui)
            green "Done. Try:  $GUI_BIN --help"
            green "dd_siteforge-gui needs Vulkan and Wayland or X11."
            ;;
        both)
            green "Done. Try:  $TUI_BIN --help    or    $GUI_BIN --help"
            green "dd_siteforge-gui needs Vulkan and Wayland or X11."
            ;;
    esac
}

install_built() {
    local name=$1 src_bin
    src_bin="target/release/$name"
    [ -x "$src_bin" ] || { red "Build did not produce $src_bin"; exit 1; }
    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$name"
    green "Installed $name → $BIN_DIR/$name"
}

# ---- subcommands ------------------------------------------------------------
do_install_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        red "Required command not found: cargo"
        yellow "Hint: download a prebuilt binary with  ./install.sh --from-release"
        yellow "  or: curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash"
        exit 1
    fi

    local root
    root="$(repo_root)"
    cd "$root"

    [ -f Cargo.toml ] || { red "No Cargo.toml in $root — run install.sh from the repo root, or omit --from-source."; exit 1; }
    [ -f "$THEME_FILE" ] || { red "Missing $THEME_FILE in $root."; exit 1; }

    case "$INSTALL_TARGET" in
        tui)
            cyan "Building $TUI_BIN (release)…"
            cargo build --release
            install_built "$TUI_BIN"
            install_theme_if_absent "$THEME_FILE"
            ;;
        gui)
            cyan "Building $GUI_BIN (release)…"
            cargo build --release -p dd_siteforge_gui
            install_built "$GUI_BIN"
            ;;
        both)
            cyan "Building $TUI_BIN (release)…"
            cargo build --release
            install_built "$TUI_BIN"
            cyan "Building $GUI_BIN (release)…"
            cargo build --release -p dd_siteforge_gui
            install_built "$GUI_BIN"
            install_theme_if_absent "$THEME_FILE"
            ;;
        *) red "internal: unknown target $INSTALL_TARGET"; exit 1 ;;
    esac
    install_launchers
    finish_note
}

fetch_release() {
    local name=$1 triple=$2 tmp=$3
    local asset archive_url sum_url archive src_bin
    asset="$(asset_base "$name" "$triple").tar.gz"
    archive_url="$(release_url "$asset")"
    sum_url="$(release_url "$asset.sha256")"

    if [ -n "$VERSION" ]; then
        cyan "Downloading $name $VERSION ($asset)…"
    else
        cyan "Downloading $name (latest $asset)…"
    fi

    archive="$tmp/$asset"
    if ! http_get "$archive_url" "$archive"; then
        red "Failed to download:"
        red "  $archive_url"
        red ""
        red "No prebuilt package for $triple (or that release has no assets yet)."
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

    src_bin="$tmp/$name"
    if [ ! -f "$src_bin" ]; then
        src_bin="$(find "$tmp" -type f -name "$name" ! -name '*.sha256' | head -n 1 || true)"
    fi
    [ -n "$src_bin" ] && [ -f "$src_bin" ] || { red "Archive did not contain $name"; exit 1; }
    chmod +x "$src_bin"

    mkdir -p "$BIN_DIR"
    install_file 0755 "$src_bin" "$BIN_DIR/$name"
    green "Installed $name → $BIN_DIR/$name"
}

install_theme_from_tree() {
    local tmp=$1 theme_src
    theme_src="$tmp/$THEME_FILE"
    if [ ! -f "$theme_src" ]; then
        theme_src="$(find "$tmp" -type f -name "$THEME_FILE" | head -n 1 || true)"
    fi
    install_theme_if_absent "${theme_src:-}"
}

do_install_from_release() {
    normalize_version
    if [ -n "$TARGET" ] && [ "$INSTALL_TARGET" = both ]; then
        yellow "TARGET is ignored for --both. The terminal app uses musl and the window uses glibc."
    fi

    local tmp
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    case "$INSTALL_TARGET" in
        tui)
            fetch_release "$TUI_BIN" "$(resolve_triple tui)" "$tmp"
            install_theme_from_tree "$tmp"
            ;;
        gui)
            fetch_release "$GUI_BIN" "$(resolve_triple gui)" "$tmp"
            ;;
        both)
            fetch_release "$TUI_BIN" "$(resolve_triple tui)" "$tmp"
            install_theme_from_tree "$tmp"
            fetch_release "$GUI_BIN" "$(resolve_triple gui)" "$tmp"
            ;;
        *) red "internal: unknown target $INSTALL_TARGET"; exit 1 ;;
    esac
    install_launchers "$tmp"
    finish_note
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
    local target=$1
    local removed=0 path
    local -a paths=()
    case "$target" in
        tui)
            paths=(
                "$BIN_DIR/$TUI_BIN"
                "$(apps_dir)/${TUI_BIN}.desktop"
            )
            ;;
        gui)
            paths=(
                "$BIN_DIR/$GUI_BIN"
                "$(apps_dir)/${GUI_BIN}.desktop"
            )
            ;;
        both)
            paths=(
                "$BIN_DIR/$TUI_BIN"
                "$BIN_DIR/$GUI_BIN"
                "$(apps_dir)/${TUI_BIN}.desktop"
                "$(apps_dir)/${GUI_BIN}.desktop"
            )
            ;;
        *) red "internal: unknown target $target"; exit 1 ;;
    esac
    for path in "${paths[@]}"; do
        if [ -e "$path" ] || [ -L "$path" ]; then
            rm -f -- "$path"
            green "Removed $path"
            removed=1
        fi
    done
    local apps
    apps="$(apps_dir)"
    if command -v update-desktop-database >/dev/null 2>&1 && [ -d "$apps" ]; then
        update-desktop-database "$apps" >/dev/null 2>&1 || true
    fi
    if [ "$removed" -eq 0 ]; then
        yellow "No matching binaries or launchers found."
    else
        green ""
        green "Uninstall complete."
    fi
    local session_dir="$CONFIG_DIR/dd_siteforge"
    if [ -d "$session_dir" ]; then
        yellow "Left $session_dir in place (session and scaffold)."
    fi
    yellow "Left the theme file and the icon in place."
}

# ---- dispatch ---------------------------------------------------------------
main() {
    print_mark
    local action=install
    local mode=auto
    local target=""
    while [ $# -gt 0 ]; do
        case "$1" in
            install) action=install; shift ;;
            uninstall|remove|--uninstall)
                action=uninstall
                shift
                ;;
            --from-source) mode=source; shift ;;
            --from-release|--binary)
                mode=release
                shift
                ;;
            --tui|--gui|--both)
                if [ -n "$target" ]; then
                    red "pass only one of --tui, --gui, or --both"
                    exit 1
                fi
                target=${1#--}
                shift
                ;;
            --version)
                [ $# -ge 2 ] || { red "--version needs a value (e.g. v1.32.0)"; exit 1; }
                VERSION="$2"
                shift 2
                ;;
            --version=*) VERSION="${1#*=}"; shift ;;
            --target)
                [ $# -ge 2 ] || { red "--target needs a rust triple"; exit 1; }
                TARGET="$2"
                shift 2
                ;;
            --target=*) TARGET="${1#*=}"; shift ;;
            -h|--help|help) usage; exit 0 ;;
            *)
                red "Unknown argument: $1"
                usage
                exit 1
                ;;
        esac
    done
    if [ -z "$target" ] && [ "${INSTALL_GUI:-}" = 1 ]; then
        target=both
    fi
    INSTALL_TARGET=$(choose_target "$action" "$target")

    if [ "$action" = uninstall ]; then
        do_uninstall "$INSTALL_TARGET"
        return 0
    fi
    if [ "$mode" = source ] && ! in_repo; then
        red "--from-source needs a clone of the repo (Cargo.toml next to install.sh)."
        red "To install a prebuilt binary:"
        red "  curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash"
        exit 1
    fi
    do_install "$mode"
}

# Run when executed as a file or via `curl | bash`. Skip when sourced.
# Under `set -u`, a pipe leaves BASH_SOURCE[0] unset (or set to bash / a fd path).
case "${BASH_SOURCE[0]:-$0}" in
    "$0"|bash|sh|-|/dev/fd/*|/proc/self/fd/*) main "$@" ;;
esac
