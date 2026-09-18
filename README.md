# dd_siteforge

Terminal-UI CMS for authoring framework-native static pages. Single Rust binary: edit a typed site tree, export HTML, host anywhere static.

**Tutorial (setup, TUI walkthrough, screenshots):** open [`docs/tutorial/index.html`](docs/tutorial/index.html).

## Install

Picks the Linux or macOS binary for this machine (x86_64 or aarch64) from GitHub Releases:

```bash
curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash
```

Installs `$HOME/.local/bin/dd_siteforge` and writes the default theme to `$HOME/.config/ldnddev/dd_siteforge_theme.yml` only when that file is missing. Pin a version with `--version`, or override `PREFIX` / `BIN_DIR` / `CONFIG_DIR`.

```bash
curl -fsSL https://raw.githubusercontent.com/ldnddev/dd_siteforge/master/install.sh | bash -s -- --version v1.11.0
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
npm install && npx grunt build
dd_siteforge tui site.json
```

In the TUI: `F1` help, `Shift+E` export, `p` preview, `Ctrl+Q` quit. Put images in `./source/images/`.

## Tests

```bash
cargo test -q
```

## License

MIT License.
