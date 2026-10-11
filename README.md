# dd_siteforge

Terminal-UI CMS for authoring framework-native static pages. Single Rust binary: edit a typed site tree, export HTML, host anywhere static.

**Tutorial (setup, TUI walkthrough, screenshots):** [ldnddev.github.io/dd_siteforge](https://ldnddev.github.io/dd_siteforge/).

## Install

Picks the Linux or macOS binary for this machine (x86_64 or aarch64) from GitHub Releases:

```bash
curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash
```

Installs `$HOME/.local/bin/dd_siteforge` and writes the default theme to `$HOME/.config/ldnddev/dd_siteforge_theme.yml` only when that file is missing. Pin a version with `--version`, or override `PREFIX` / `BIN_DIR` / `CONFIG_DIR`.

```bash
curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash -s -- --version v1.26.0
curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash -s -- uninstall
```

From a clone, `./install.sh` builds with cargo. Use `--from-release` to download a binary instead.

```bash
./install.sh                      # cargo build --release
./install.sh --from-release       # same as the curl one-liner
cargo install --path .            # ~/.cargo/bin
./install.sh uninstall
```

## Quick start

```bash
dd_siteforge init-site site.json --name my-site
lando start && lando npm install && lando grunt build
dd_siteforge
```

`dd_siteforge` in a folder with `site.json` opens the TUI. `dd_siteforge site.json` and `dd_siteforge tui site.json` still work. Recents and last page/tree row live in `~/.config/ldnddev/dd_siteforge/session.json`.

In the TUI: `F1` help, `?` find, `:` command palette, `Shift+E` export, `Shift+P` preview, `Shift+B` build CSS/JS, `Ctrl+Q` quit. Put images in `./source/images/`.

## Tests

```bash
cargo test -q
```

## License

MIT License.
