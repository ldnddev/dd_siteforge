# dd GUI template

**Portable contract for a GPUI window beside a dd_* terminal app.**

Copy this file into a new repo as the window contract. The document surface belongs to the app (pages, notes, files, vaults). The workspace shape, the shared chord table, the Settings sheet, the theme, the chrome that stays put, and the installer are shared.

This guide was taken from the `dd_siteforge` window after it was brought in line with the `dd_vault` settings sheet. Where the two apps agree, follow that agreement. Where an app needs a panel the others do not have, add it inside the same shell. Do not invent a second settings screen.

---

## Required vs optional

| Required in every window | Optional (only if the app needs it) |
|---|---|
| A `{app}-gui` binary next to the terminal binary | A second companion window |
| `ldnddev_theme` and the same `{app}_theme.yml` the terminal uses | Density (Comfortable / Compact) |
| One command table in the core crate, read by both frontends | Vim letters |
| F1 Settings: one right-half sheet, theme first, shortcuts below | A command palette and a find sheet |
| Outlined sections with a gap between boxes | Docks (sidebar, inspector) |
| Window prefs in `gui.json`, never in the user's document | An issues or health drawer |
| `install.sh --tui`, `--gui`, and `--both` | Release downloads and checksums |
| `gpui::test` on the window binary | A block-art wordmark in `install.sh` |

---

## How to adopt

1. Copy this file into the new repo.
2. Name the app `dd_{name}`. The terminal binary is `dd_{name}`. The window binary is `dd_{name}-gui`. The app id is `dev.ldnddev.dd_{name}`.
3. Put the document model, validation, and save path in `crates/dd_{name}_core`. Both frontends call that crate. The window does not grow a second model.
4. Put the GPUI shell in `crates/dd_{name}_gui`. Leave that crate out of `default-members` so a plain `cargo build` and a terminal-only install do not compile GPUI.
5. Depend on `ldnddev_theme`. Load `{app}_theme.yml` in the same order as the terminal. Resolve colors once. Do not blend them in the shell.
6. Put every chord in the core command table. The window adapter only turns a key token into a chord and runs the command.
7. Build the shell chrome first: title bar, the document surface, then the F1 Settings sheet. Open Settings and confirm the editor underneath does not reflow.
8. Wire `install.sh` to the choice model below before the first time anyone installs the window.
9. Lock the sheet with dump lines in a `gpui::test` (`settings-panel:right:70`, `settings-nav:…`, `settings-page:…`). Pixels are not the contract.

---

## Workspace

```
Cargo.toml                  terminal binary, default member
crates/ldnddev_theme/       shared theme crate, no ratatui, no gpui
crates/dd_{name}_core/      model, commands, save
crates/dd_{name}_gui/       window binary dd_{name}-gui
```

`Cargo.toml` lists the GUI crate in `members` and leaves it out of `default-members`. CI and `cargo deny` still see it. `cargo build --release -p dd_{name}_gui` is the window build.

Pin the GPUI stack. The reference window uses:

- `gpui` package `gpui-pre`, version `=0.3.12`
- `gpui_platform` package `gpui-pre-platform`, version `=0.3.12`, features `wayland` and `x11`
- `gpui_macros` package `gpui-pre-macros`, version `=0.3.12`
- `gpui-component` version `=0.9.1`

The release profile is `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.

The window crate is a binary, not a library. Tests are `cargo test -p dd_{name}_gui --offline --bin dd_{name}-gui`. `cargo test -p dd_{name}_gui --lib` has nothing to run.

---

## Open the window

`main` starts the platform, initializes `gpui-component`, and opens one window into `Root`:

```rust
gpui_platform::application().run(move |cx: &mut App| {
    gpui_component::init(cx);
    cx.spawn(async move |cx| {
        cx.update(|cx| {
            let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
            cx.open_window(main_window_options(bounds), |window, cx| {
                let view = cx.new(|cx| {
                    let shell = Shell::open(ShellOptions::production(path), cx);
                    window.focus(&shell.focus_handle(cx), cx);
                    shell
                });
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("open window");
        });
    })
    .detach();
});
```

`WindowOptions.app_id` is `dev.ldnddev.dd_{name}`. On Wayland that is the xdg app id. On X11 it is the window class. A companion window (preview, or any second surface) gets its own id, `dev.ldnddev.dd_{name}.preview`, so a desktop rule can target it alone.

Opening another window while `Shell` is already being updated panics with `cannot read Shell while it is already being updated`. Any path that opens a window from inside a shell update goes through `cx.defer`. The same rule covers settling a companion after the shell frame.

The shell owns screen state, the open document, the theme, and which sheet is up. A `present` module (or the same split under another name) builds the rows the shell paints. Render and the tests both read those rows. Do not paint from the core model directly inside `Render`.

---

## One command table

`crates/dd_{name}_core/src/commands.rs` is the only binding list.

- A `Chord` is key + control + alt + shift. `Chord::parse` accepts the window's token form (`f1`, `ctrl-/`, `ctrl-shift-n`, `ctrl-\`).
- `lookup` filters by surface (window or terminal), keyboard mode (Kitty, legacy, tmux), whether Vim is on, and whether a text field is focused.
- `command_for_chord` is what the window calls. A chord the window treats as primary has `BindingKind::Primary` in the table. An alias rank will fail the test that every window primary chord is in the shared table.
- The window crate is a thin adapter: token in, `CommandId` out. Vim letters and single-key verbs stay in the adapter. They are not command chords, and they do nothing while a text field is focused.
- The palette is a filtered view of the same specs (`in_palette`). The terminal palette uses `tui_palette`. Titles and details are the spec strings.
- `shortcut_sheet()` is a static list of `SheetRow { section, action, gui, tui }`. Both apps render that list. The window column is the GUI key. The terminal column is the TUI key.

Settings is one command. The spec title is `Settings`. The chords are `f1`, `ctrl-/`, `f2`, and `ctrl-,`, all primary. The kbd label is `F1`. The sheet's first Global row is `Settings · F1 · F1`.

`Command::Theme` stays in the enum so older call sites still compile. It has no chords, an empty kbd, and both palette flags false. Dispatching it opens the same sheet as Help. The sheet and the footer advertise F1.

F3, F4, and the rest of the function keys stay free for the app (validate, health, and so on). Add them to the same table and the same sheet.

---

## The Settings sheet

F1 opens one sheet. F2, Ctrl+/, and Ctrl+, open and close that same sheet. The footer reads `F1: Settings`. There is no second theme screen.

### Geometry

The window sheet is 70% of the body band, under the header and above the footer. The terminal sheet is the right half of the body. Neither covers the editor with a full-screen scrim, so F3 or F4 do not reflow the window.

```text
id("settings-panel")
  occlude
  absolute
  top = header height
  right = 0
  bottom = footer height
  width = 70%
  border left 1px
  click stops propagation
```

The header is a 48px row: the word `Settings`, a flexible gap, the label `F1`, and `Close`. Under that, a 200px nav (Search, then Themes, Window, Help) and one page. The page is one scroll, `id("settings-scroll")`.

While the sheet is open, `handle_key` returns into the sheet's key function and does not reach the normal command runner. An open quit or restore confirm is handled first. F1, F2, Ctrl+/, and Ctrl+, close the sheet even while a hex field is focused.

### What the sheet contains

One page at a time. Tab, Shift-Tab, `[`, and `]` change the page. `t` toggles the save target. `/` focuses search. Esc clears a search before it closes the sheet.

1. **Themes.** Omarchy names, the source and status line, Save to (Global / Local), Edit colors, Save.
2. **Window.** Density (Comfortable or Compact) and the Vim switch.
3. **Help.** The shortcut sheet. A note (`GUI key, then TUI key. F1 opens and closes this panel.`), then one outlined group per `SheetRow.section`.

The color editor replaces the theme list with a **Colors** box. Escape there steps back to the list when a list exists. The next Escape reverts the theme snapshot taken when the sheet opened and closes the sheet. Close does the same.

### How a section is drawn

Each box is a 1px outline in `border.subtle`. The header band is 28px, bold, `surface.raised`, `text.primary`, with a 1px bottom border. The body is padded 8px.

Shortcut groups (Global, and whatever sections the app adds) are the same kind of box, nested inside Keyboard shortcuts. That is the Provider-inside-AI pattern from the vault sheet: a list of choices gets its own outline so it does not run into the next group.

On the Themes page the names are the first children of `settings-scroll`, so a selected name can be revealed:

| Child index | What it is |
|---|---|
| n | Theme name *n*, 28px, `overflow_hidden` |
| after the names | Source line, then the actions |

`ScrollHandle::scroll_to_item(selected)` runs after the selection moves. The lead count is 0. If you insert a child above the names, update that count.

Do not put the names in a nested scroll with a max height. A `max_h` plus `overflow_y_scroll` inside `settings-scroll` does not clip. The names paint through the bottom rule and over the source line. One scroll, fixed-height rows, `overflow_hidden` on each row.

### Theme rows behave like the vault sheet

- Clicking a name selects it and applies it.
- Up, Down, Page Up, Page Down, and (when Vim is on) j, k, g, and G move the highlight and apply it.
- Enter applies the highlighted name again.
- The apply writes into the live palette. The snapshot from when the sheet opened stays. Escape restores that snapshot.
- `y` or `s` writes the palette to the save target. `t` toggles Global / Local. Tab changes the page.
- `e` opens the color editor.
- A missing theme file or a failed Omarchy read falls back to a built-in palette (retro-82 in the reference app) and shows the warning on the source line. The window still opens.

---

## Theme and window prefs

`ldnddev_theme` loads the YAML, maps an Omarchy `colors.toml`, and resolves a `GuiTheme`. The shell asks the theme crate for colors. It does not parse the YAML itself.

Lookup order, same as the terminal:

1. `./{app}_theme.yml` (local)
2. `$XDG_CONFIG_HOME/ldnddev/{app}_theme.yml`, or `~/.config/ldnddev/{app}_theme.yml` (global)
3. Built-in defaults

`Follow` tracks the desktop theme. Re-read it on a short timer while the sheet is closed. Skip the re-read while the sheet is open so a live preview is not overwritten.

Window prefs are a separate file, `{config}/dd_{name}/gui.json`:

```json
{
  "density": "comfortable",
  "vim_keys": true,
  "reduced_motion": false
}
```

`vim_keys` defaults to true. `reduced_motion` defaults to false. With reduced motion, theme apply is instant (`theme-apply` duration 0). These keys never appear in the user's document (`site.json`, the vault, the note). A round-trip test writes `gui.json` in a temp dir and checks the document file was not touched.

Map paint to tokens (`surface.base`, `surface.panel`, `surface.raised`, `text.primary`, `text.secondary`, `border.subtle`, `selection.bg`, `accent.fill`). After load, the shell does not hard-code colors.

---

## Chrome that stays put

The title bar stays visible with the sheet open. Search, when the app has it, lives in the title bar (a fixed width, 240px in the reference window). Export and Preview, when the app has them, are title-bar actions. Init and other rare commands stay in the palette.

Docks, when the app has them:

- The sidebar is visible above 720px, and only when pinned at 720px and below.
- The inspector is visible from 1000px up, and only when pinned below that.
- The title-bar toggles and the chords (`Ctrl+\` for the sidebar, `Ctrl+Alt+\` for the inspector in the reference app) call the same functions. Pick chords in the shared table and use them in both places.

Density changes spacing one notch (row, input, button, padding, gaps). Type size, tree indent, and dock widths stay. Comfortable rows are 28px. Compact rows are 24px.

Opening Settings, the palette, or find clears the other sheet. One sheet at a time. The document surface underneath keeps its scroll, selection, and open drawers.

---

## Other overlays

The palette and find, when the app has them, are centered sheets with their own widths. They are not the Settings panel. Escape closes them. They do not change dock widths.

An enum in the editor is a closed control that shows the value and a `▾`. Click, Left, or Right opens an in-panel list, capped at about eight rows (`metrics.input * 8.0_f32`, and the list needs an element id before `overflow_y_scroll`). Choosing a row writes the field and closes the list. Up and Down move inside the open list. Escape closes the list and leaves the draft. Tab closes the list and moves focus. Use the same control for every enum. A toggle stays a switch.

Clicks on a sheet call `cx.stop_propagation()` so the editor underneath does not take the click.

---

## Installer

`install.sh` asks which program to install. The flags match across apps.

| Flag | What it installs |
|---|---|
| `--tui` | The terminal binary only |
| `--gui` | The window binary only |
| `--both` | Both binaries and both launchers |
| (no flag, with a tty) | Prompt: 1 terminal, 2 window, 3 both. Default is 1 on install and 3 on uninstall |
| (no flag, no tty) | Install the terminal app. Uninstall removes both |
| `INSTALL_GUI=1` | Same as `--both` |

`--tui` and `--gui` together is an error. Pass one of `--tui`, `--gui`, or `--both`.

The prompt is written to `/dev/tty` and the answer is read from `/dev/tty`. A failed tty redirect inside a captured command must not abort the script (`set -e`).

Uninstall removes the chosen binaries and their `.desktop` files. It leaves the theme YAML, the icon, and `$CONFIG_DIR/dd_{name}` (session and any scaffold). Say that in the usage text.

`data_dir` prefers `$XDG_DATA_HOME` over `PREFIX`. A dry run that sets only `PREFIX` still edits `~/.local/share` when `XDG_DATA_HOME` points there. A test uninstall sets `XDG_DATA_HOME` to a throwaway directory.

Linux release archives, when the app publishes them:

- The terminal is `{arch}-unknown-linux-musl`.
- The window is `{arch}-unknown-linux-gnu`.
- `--both` downloads both archives. `TARGET` is ignored in that mode, with a warning. One triple cannot build both binaries.
- Each archive has a `.sha256`. Refuse a missing or mismatched checksum.

From a clone, `--from-source` runs `cargo build --release` for the terminal and `cargo build --release -p dd_{name}_gui` for the window. `--both` runs both. The theme file is installed for the terminal and for `--both`. A window-only install does not replace the theme.

Desktop files, one per binary that exists after install:

- `dd_{name}.desktop` with `Terminal=true`
- `dd_{name}-gui.desktop` with `Terminal=false` and `StartupWMClass=dev.ldnddev.dd_{name}`

Share one icon. Refresh a launcher for every binary still present in `BIN_DIR`, so reinstalling the terminal does not drop the window launcher.

When stdout is a terminal, print the ldnddev wordmark as half-block art before the prompt or the usage text. Skip it when stdout is not a terminal, so a pipe stays plain text. Draw the mark in bright white on a black background (`\033[40;97m`) and reset after it.

---

## Tests

Window tests use `gpui::test` and a headless mount. Mounting does not draw a frame. Assert on a text dump the shell already builds (`visible_text` / `lines`), not on pixels.

Dump lines that lock this contract:

- `settings-panel:right:70` while the sheet is open
- `settings-sections:framed` while the sheet is open
- `theme-list:flow` while the theme names are in the one scroll
- The shortcut sheet text, including `Settings · F1 · F1`
- Density and Vim as they are after a change, including after the sheet is closed and opened again

A starter document that is missing assets still has validation errors. Do not assert a clean validation list unless the fixture is clean.

Key tests call `handle_key("f1")` and `handle_key("f2")` and expect the same sheet. Escape reverts. A second F1 closes.

Run:

```bash
cargo test -p dd_{name}_core --offline --lib
cargo test -p dd_{name}_gui --offline --bin dd_{name}-gui
cargo test -p dd_{name} --offline --bin dd_{name}
cargo fmt -p dd_{name} -p dd_{name}_gui -p dd_{name}_core
bash -n install.sh
```

The installed binaries are a separate step. Tests do not replace `~/.local/bin/dd_{name}-gui`. Tell the user to run `./install.sh --gui` or `./install.sh --both` when they want the new window.

---

## What stays in the app

Keep these out of the shared contract:

- The document model and its file format
- The canvas (tree, outline, blueprint, editor fields)
- Which extra commands exist beyond Settings, save, quit, and the docks
- Copy in the shortcut sheet beyond the columns the table already provides
- Preview, export, and build, unless that app actually does those jobs

When a pattern repeats in a second app (a new sheet, a new dock rule, a new installer flag), add it here first, then implement it. An app-only layout stays in that app's `present` module and its tests.
