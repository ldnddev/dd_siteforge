# dd GUI template

**Portable contract for a GPUI window beside a `dd_*` terminal app.**

Copy this file into a new repo and follow it. The document surface belongs to the app (pages, notes, files, vaults). The workspace shape, the shared behavior, the Settings sheet, the theme, the chrome, and the installer are shared.

This file is the union of the `dd_vault` window and the `dd_siteforge` window. Where those two already agree, the sections below are the rule. Where they still differ, **Current forks** names both, and the rule in the section is what a new app implements. Do not mix the two GPUI stacks in one binary.

When a pattern shows up in a second app, add it here first, then implement it.

`{app}` is the binary stem, including the `dd_` prefix (`dd_vault`, `dd_siteforge`).

---

## Required vs optional

| Required in every window | Optional (only if the app needs it) |
|---|---|
| A `{app}-gui` binary next to the terminal binary | A second companion window |
| Vendored `ldnddev_theme` and the same `{app}_theme.yml` the terminal uses | Density (Comfortable / Compact) |
| One binding list both frontends read | Vim letters |
| F1 Settings: left nav, one page, Themes first, Help last | A command palette and a find sheet |
| Outlined sections with a 12px gap | Docks (sidebar, inspector) |
| Window prefs outside the user's document | Editor font pickers and a shared `[gui]` face table |
| `install.sh --tui`, `--gui`, and `--both` | Release downloads, checksums, and a block-art wordmark |
| Tests that run without a display | `gpui::test` dump lines, when that stack has test support |
| `panic = "abort"` and `strip = true` | An issues or health drawer |

---

## How to adopt

1. Copy this file into the new repo.
2. Name the terminal binary `{app}` and the window binary `{app}-gui`. The app id is `dev.ldnddev.{app}`.
3. Put files, config, session, validation, and save in the core crate. Both frontends call that crate. The window does not grow a second model and does not depend on the terminal crate.
4. Pick one workspace profile and one GPUI stack from the tables below. Pin that stack with `=`.
5. Depend on the in-repo `ldnddev_theme`. Load `{app}_theme.yml` in the same order as the terminal. Resolve colors once.
6. Put every chord in the shared binding list. The window adapter turns a key into that list's key type and runs the command.
7. Build the shell chrome first: title bar, body, footer, then the F1 Settings sheet. Open Settings and confirm the editor underneath keeps its place.
8. Wire `install.sh` to the choice model below before the first time anyone installs the window.
9. Lock the sheet with tests. On the `gpui-pre` stack those are dump lines (`settings-panel:right:70`, `settings-nav:…`, `settings-page:…`, and `theme-list:flow` on the Themes page). On the `gpui-unofficial` stack they are plain functions over row height, `content_h`, and the revert snapshot. Pixels are not the contract. Then exercise the changed path in a real window.

---

## Workspace

The theme crate lives inside the repo. The window path-depends on `../ldnddev_theme`. It does not path-depend on another checkout, and it does not depend on another app's GUI crate.

```
{app}/
  Cargo.toml                     terminal workspace
  crates/ldnddev_theme/          vendored, no ratatui, no gpui
  crates/{app}_core/             files, config, session, domain rules
  crates/{app}_gui/              window binary {app}-gui
```

A terminal-only crate (`{app}_tui`) and shared editor or render crates are added when both frontends need them. Domain modules sit beside the shell. A preview, a git badge, or a command palette belongs in the window only when the terminal already has that behavior.

Two profiles are legal. Pick one.

| | Profile A, one workspace | Profile B, window workspace |
|---|---|---|
| Who uses it | The whole repo already builds on a Rust that can compile GPUI (`dd_siteforge`) | The terminal must stay on an older rust-version (`dd_vault`: edition 2021, Rust 1.82) |
| GUI membership | Listed in `members`, omitted from `default-members` | Not a member. Its own `Cargo.toml`, `Cargo.lock`, and `deny.toml` |
| Window edition | Same as the rest of the repo | Edition 2024, `rust-version = "1.92"` |
| Build | `cargo build --release -p {app}_gui` | `cargo build --release --manifest-path crates/{app}_gui/Cargo.toml` |
| Test | `cargo test -p {app}_gui --offline --bin {app}-gui` | `CARGO_TARGET_DIR=target/{app}_gui cargo test --manifest-path crates/{app}_gui/Cargo.toml` |
| Why the split | `cargo build` and a terminal-only install skip GPUI | A shared `target/` makes the two cargo locks wait on each other. Give the window its own target directory |

`cargo test --workspace` from the repo root must keep passing on the terminal's rust-version. CI and `cargo deny` still see the window. Profile A does that by membership. Profile B runs `cargo deny check` inside `crates/{app}_gui`.

If both UIs edit text, the buffer and the caret live in one editor crate with no ratatui types. The window maps keystrokes onto that crate's `Key`. If both UIs draw a document, one render crate produces the blocks. The terminal draws cells. The window lays the same blocks out in pixels.

The window crate is a binary. `cargo test -p {app}_gui --lib` has nothing to run on Profile A.

---

## Pins

Pin GPUI with `=`. A floating upgrade changes text layout and input. Leave Zed git dependencies, xgpui, fc-gpui, and guic out of the crate.

GPL stays off the allow list. Apache-2.0 is allowed because GPUI is Apache-2.0. Unknown git sources are denied. Advisories that ship inside the pinned GPUI are ignored with a reason that names the pin. Profile A uses the workspace `deny.toml`. Profile B uses a `deny.toml` in the window crate, separate from the terminal workspace.

The terminal release profile stays small: `opt-level = "z"`, fat LTO, `codegen-units = 1`, `panic = "abort"`, `strip = true`. Every window also sets `panic = "abort"` and `strip = true`.

### Stack S, `gpui-pre` (`dd_siteforge` today)

```toml
gpui = { package = "gpui-pre", version = "=0.3.12" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.12", features = ["wayland", "x11"] }
gpui_macros = { package = "gpui-pre-macros", version = "=0.3.12" }
gpui-component = "=0.9.1"
```

Dev tests add `gpui` with the `test-support` feature. The window inherits the workspace release profile (`opt-level = "z"`, fat LTO, one codegen unit). `main` calls `gpui_component::init` and opens the window into `Root`.

### Stack U, `gpui-unofficial` (`dd_vault` today)

```toml
gpui-unofficial = "=1.27.0"
gpui_platform = { package = "gpui-platform-gpui-unofficial", version = "=1.27.0", features = ["wayland", "x11"] }
```

The window release profile is `opt-level = 3`, `lto = "thin"`, `codegen-units = 16`, `panic = "abort"`, `strip = true`. There is no `gpui-component` and no `Root` wrapper. The shell entity is the window root.

The window needs Vulkan, Wayland or X11, fontconfig, and freetype. Say that in the installer. `init` and any no-window subcommand still run without a display.

---

## Open the window

`main` parses the same CLI as the terminal. Subcommands that only touch disk print and return. They do not open a window. The no-argument path, and any open-a-document path, load the theme and the shared paths, then start the shell.

```rust
pub fn run(boot: Boot) {
    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("{app}".into()),
                    ..Default::default()
                }),
                focus: true,
                show: true,
                app_id: Some("dev.ldnddev.{app}".into()),
                ..Default::default()
            },
            move |window, cx| {
                let shell = cx.new(|cx| Shell::new(boot, cx));
                shell.update(cx, |this, cx| this.focus.focus(window, cx));
                shell
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
```

On Stack S the view returned to `open_window` is `Root::new(...)` after `gpui_component::init`. The bounds above are the siteforge window (1280×800). Vault opens at 1100×720. A new app uses 1280×800.

`WindowOptions.app_id` is `dev.ldnddev.{app}`. On Wayland that is the xdg app id. On X11 it is the window class. The Linux desktop file uses that same string as `StartupWMClass`. A companion window gets its own id, `dev.ldnddev.{app}.preview`, so a desktop rule can target it alone.

Opening another window while `Shell` is already being updated panics with `cannot read Shell while it is already being updated`. Any path that opens a window from inside a shell update goes through `cx.defer`. The same rule covers settling a companion after the shell frame.

One `Shell` entity owns the screen, the open document, the theme, and which sheet is up. `Render` keeps a `FocusHandle` on the root and focuses it whenever the window is not focused. Keys arrive on that root through `on_key_down`.

A `present` module, when the app has one, builds the rows the shell paints. Render and the tests both read those rows. Paint goes through those rows, or through the shared render crate. It does not re-derive the document inside `Render`.

---

## One command table

Both frontends read one binding list. In a single-core app that list is `crates/{app}_core/src/commands.rs`. Where the editor crate owns `Key`, the window maps a GPUI keystroke onto that enum in `keys.rs` and the rest of the shell matches on the enum.

- A chord is key + control + alt + shift. The parser accepts the window's token form (`f1`, `ctrl-/`, `ctrl-shift-n`, `ctrl-\`).
- Lookup filters by surface (window or terminal), keyboard mode (Kitty, legacy, tmux), whether Vim is on, and whether a text field is focused.
- A chord the window treats as primary has `BindingKind::Primary`. An alias rank fails the test that every window primary chord is in the shared table.
- The window crate is a thin adapter: token in, command out. Vim letters and single-key verbs stay in the adapter. They do nothing while a text field is focused.
- The palette is a filtered view of the same specs. Titles and details are the spec strings.
- `shortcut_sheet()` is a static list of `SheetRow { section, action, gui, tui }`. Both frontends render that list. The first column is the GUI key. The second is the TUI key.

Settings is one command. The spec title is `Settings`. The chords are `f1`, `ctrl-/`, `f2`, and `ctrl-,`, all primary. The kbd label is `F1`. The sheet's first Global row is `Settings · F1 · F1`. The footer reads `F1: Settings`.

`Command::Theme`, when the enum still has it, has no chords, an empty kbd, and both palette flags false. Dispatching it opens the same sheet. The sheet and the footer advertise F1.

F3, F4, and the rest of the function keys stay free for the app (validate, health, and so on). Add them to the same table and to the shortcut sheet. They are app commands. They are not a second settings window.

The command palette's Settings item and its Help item open this sheet. Help scrolls to the shortcut section. Opening Help does not call itself.

---

## Input

Handle a key in this order:

1. Ctrl+V. Paste text, or an image when the focused surface accepts one.
2. An open confirm, even while Settings is up. Settings must not swallow a quit prompt.
3. The Settings sheet, when it is open. F1, F2, Ctrl+/, and Ctrl+, are handled here, before the color editor, so they close the sheet even while a hex field is focused.
4. F1, when nothing modal is up. This path only opens.
5. Ctrl+Q and Ctrl+S.
6. The focused pane, the leader, or the command palette.

Closing on F1 happens inside the sheet handler. The opener only opens. One path that both opens and closes rebuilds the theme picker and drops the palette the user would revert to. Re-entering the sheet while it is already open returns immediately after revert. Building a new picker on that path replaces the revert snapshot.

Ctrl+Q quits. A bare `q` does not. A dirty document confirms before quit. Esc cancels the top modal (confirm, color editor, open enum list), then clears a search, then closes the sheet and reverts the theme snapshot taken when the sheet opened.

Named keys (`up`, `escape`, `backspace`) become the shared key enum. Control plus a letter becomes `Ctrl(char)`.

A one-line field (rename, prompt, search, hex, filter) keeps a caret in the real text. Left, Right, Home, End, and a click move it. Backspace deletes before the caret. Delete deletes the character after it. A masked field paints bullets and moves the caret in the underlying text. Up and Down stay list motion when the field is attached to a list. A lone field uses Up for the start and Down for the end.

Paste inserts the whole clipboard in one editor operation. It does not walk the text as typed characters. Bracketed paste and Ctrl+V call that same insert. Sanitize the string in the editor crate before it lands. An image paste writes a file through the core's confined write and inserts one link, when the focused surface accepts an image.

Clicks hit the deepest element first. A row or a sheet that handles the click calls `stop_propagation`. The shell root still records caret hit boxes for fields painted inside that root, so a click in a search field or a hex field lands on a character. A click on padding does nothing.

The wheel on a pane scrolls that pane. It does not also move the keyboard selection. Parent `on_scroll` and a child scroller both observe the wheel. If the child owns the scroll, the parent returns without a second nudge.

An enum in the editor is a closed control that shows the value and a `▾`. Click, Left, or Right opens an in-panel list, capped at about eight rows (`metrics.input * 8.0_f32`). The list needs an element id before `overflow_y_scroll`. Choosing a row writes the field and closes the list. Up and Down move inside the open list. Escape closes the list and leaves the draft. Tab closes the list and moves focus. Use the same control for every enum. A toggle stays a switch.

---

## Scroll

One scroll owns the Settings document. A `max_h` plus `overflow_y_scroll` nested inside that scroll does not clip on the `gpui-pre` stack. The rows paint through the bottom rule and over the next section. Keep fixed-height rows in the one scroll, with `overflow_hidden` on each row.

GPUI's scroll handle is `0` at the top and negative after the user scrolls down. A `ScrollPort`, on the stack that uses one, stores the positive distance and the handle:

- `push_px(y)` writes `set_offset(point(px(0.), px(-y)))` and remembers `y`.
- `pull_px` reads the handle on the next frame. It ignores a change that matches the last push, so a wheel and a key do not fight.
- `view_px` is empty until the scroller has been painted. A reveal that runs before the first paint uses a fallback viewport, about 400px.

```rust
fn y_scroll(id: &'static str, handle: &ScrollHandle, document: Div) -> Stateful<Div> {
    div()
        .id(id)
        .absolute()
        .size_full()
        .overflow_y_scroll()
        .track_scroll(handle)
        .child(document)
}
```

`overflow_y_scroll` is called after `.id(...)`. On Stack U the document's height is the sum of its rows, set with `h(px(content_h))`. The thumb uses that height, the viewport, and the positive offset. Hide the thumb when the content fits. Dragging the thumb or clicking the track calls `push_px`.

Reveal a selected row only when the selection moved by keyboard or by click. Revealing on every frame fights the wheel.

On Stack S, `ScrollHandle::scroll_to_item` reveals by direct child index. On the Themes page the names are the first children of `settings-scroll`, so the lead count is 0. Inserting a child above the names updates that count. Wrapping the names in one tall child makes the reveal jump to the top of that child.

---

## The Settings sheet

F1 opens one sheet. F2, Ctrl+/, and Ctrl+, open and close that same sheet. There is no second theme screen and no full-screen scrim, so the editor underneath keeps its scroll, its selection, and its open drawers. F3 or F4 do not reflow the window.

### Geometry

The terminal sheet is the right half of the body, between the header and the footer. The border title is ` Settings `. The left column is the nav, about 14 columns, with a right border. A row is `> Label` when that page is showing and `  Label` otherwise. The selected row uses the selection background.

The window sheet is 70% of the body band. Width is `relative(0.7)`. The title bar and the footer stay full width. The sheet starts under the header and stops above the footer.

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

The header row is pinned: the word `Settings`, a flexible gap, the label `F1`, and `Close`. Under that is a row. The nav column is 200px and starts with Search. Each page row is `> Label`, and the selected row uses the selection background. A click on a nav row selects that page and clears the search. The page column has the section title, a one-line hint, then one scroll, `id("settings-scroll")`, with `overflow_y_scroll`, `flex_1`, `min_h(0)`, and a scroll handle.

While the sheet is open the footer names F1, Esc, and quit: `F1: Settings`, `Esc: Close`, `Ctrl+Q: Quit`. Below 80 columns those strings shorten and still name F1, Esc, and quit.

Opening Settings, the palette, or find clears the other sheet. One sheet at a time. Opening Settings while it is already open closes it and reverts. The opener returns before it builds a new picker, so the revert snapshot stays the one from the first open.

### Pages

One page is visible. Tab, Shift-Tab, `[`, and `]` change the page and reset the row. A hex field keeps `[` and `]`. Tab still changes the page. The first page is Themes. The last page is Help. The pages between those belong to the app.

`dd_vault` uses Themes, Editor, Window, AI, Keymap, Help. `dd_siteforge` uses Themes, Window, Help. Window holds Density and Vim keys. Those values live in `gui.json` and never in the user's document.

Help is the shortcut sheet. On the terminal, j and k scroll that page. On the window, the shortcut groups are outlined boxes in the one scroll.

### Themes page

Omarchy names, the source and status line, Save to (Global / Local), Edit colors, and Save. The color editor replaces the name list with a Colors box. Escape there steps back to the list when a list exists. The next Escape reverts the snapshot and closes the sheet.

`t` toggles the save target. Tab does not. While a hex field is focused, `t` is a hex character. `y` or `s` writes the palette. `e` opens the color editor.

With no Omarchy themes installed, F1 opens the color editor directly.

### Search

The window pins Search at the top of the nav. `/` focuses it. A non-empty query replaces the page with Results. Escape clears the query and leaves the sheet open. The next Escape closes the sheet and reverts. Enter on a result opens that theme, that option, or Help. The terminal sheet has no search field.

Results include theme names, the app's option labels, and shortcut actions.

### How a section is drawn

On the Themes page the names are direct children of `settings-scroll`. Do not wrap them in a nested `max_h` scroller. The source line and the actions sit after the names. A page that has boxes uses a closed outline in the border token. The header band is 28px, bold, on `surface.raised` (or the stack's header color), in the primary text color, with a bottom border. The body is padded 8px. A 12px gap separates two boxes on the same page. Shortcut groups are the same kind of box.

| | Stack S, reveal by child index | Stack U, reveal by pixel |
|---|---|---|
| Themes page | Theme name rows are 28px, `overflow_hidden`, `whitespace_nowrap`, and they are the first children of the scroll | Option rows with a title and a detail are one 52px block. Bars, gaps, and headings add their heights to `content_h` |
| Other pages | Density, Keys, and shortcut groups are outlined boxes inside that page only | The same boxes, measured in pixels |
| Hit target | The name row | The whole option block, `cursor_pointer`. The detail is not a second row |

Hover uses the body background. Selection uses the selection background and the focused text color.

### Theme rows

- Clicking a name selects it and applies it.
- Up, Down, and (when Vim is on) j, k, g, and G move the highlight and apply it.
- On the terminal, Page Up and Page Down scroll the sheet and leave the row where it is.
- On the window, Page Up and Page Down move the highlighted row by about a page on a list page.
- Enter applies the highlighted name again.
- The apply writes into the live palette. The snapshot from when the sheet opened stays. Escape restores that snapshot.
- `y` or `s` writes the palette to the save target. `t` toggles Global / Local.
- `e` opens the color editor.
- A missing theme file or a failed Omarchy read falls back to the built-in palette and shows the warning on the source line. The window still opens.

`ldnddev_theme`'s picker `handle` does not apply on j or k. The sheet calls `preview_selected` after the move, or `handle(Enter)` after it has already selected the row. Enter inside the picker stays the apply key.

---

## Theme and window prefs

`ldnddev_theme` loads the YAML, maps an Omarchy `colors.toml`, and resolves a palette. The shell asks the theme crate for colors. It does not parse the YAML itself.

Lookup order, same as the terminal:

1. `./{app}_theme.yml` (local)
2. `$XDG_CONFIG_HOME/ldnddev/{app}_theme.yml`, or `~/.config/ldnddev/{app}_theme.yml` (global)
3. Built-in defaults compiled into the binary

Load is strict for a file the user wrote and lenient for the built-in text. When the YAML has an `omarchy` key, call `apply_omarchy`. A missing Omarchy theme keeps the built-in palette and surfaces the warning. Theme YAML stays colors only.

`Follow` tracks the desktop theme. Re-read it on a short timer while the sheet is closed. Skip the re-read while the sheet is open so a live preview is not overwritten.

Pack each color once. Paint with the packed theme (`rgb(...)` on Stack U, theme tokens on Stack S). After load, the shell does not hard-code colors. Numeric fallbacks exist only for a token the palette omitted.

Tokens the shell maps onto: `surface.base`, `surface.panel`, `surface.raised`, `text.primary`, `text.secondary`, `border.subtle`, `selection.bg`, `accent.fill`. Keep the editable palette beside the packed theme. The picker and the hex editor mutate the palette, then the pack builds the next theme, then the window repaints.

Per-app window prefs live in `{config}/dd_{name}/gui.json` (`$XDG_CONFIG_HOME/ldnddev/{app}/gui.json`):

```json
{
  "density": "comfortable",
  "vim_keys": true,
  "reduced_motion": false
}
```

`vim_keys` defaults to true. `reduced_motion` defaults to false. With reduced motion, theme apply is instant. These keys never appear in the user's document and never appear in the theme YAML. A round-trip test writes `gui.json` in a temp dir and checks the document file was not touched.

Faces, when the app lets the user pick them, live in the shared ldnddev config, `$XDG_CONFIG_HOME/ldnddev/config.toml`:

```toml
[gui]
mono = "DejaVu Sans Mono"
sans = "DejaVu Sans"
```

Every windowed `dd_*` app on the machine reads this same `[gui]` table. The monospace face is the editor, the lists, and the chrome. The proportional face is for reading view. Changing a face paints immediately and recomputes the caret column width. Saving merges `[gui]` and leaves every other table, including `[ai]`, in place. Defaults are DejaVu Sans Mono and DejaVu Sans. The family list comes from fontconfig.

---

## Chrome that stays put

The window is a column: header, body, footer.

| Band | Height | Role |
|---|---|---|
| Header | 28px, or the density row height | App name and the busy mark. Search, when the app has it, is a fixed 240px field in this bar. Export and Preview, when the app has them, are actions here. |
| Body | the rest | The working panes. |
| Footer | 28px, or the density row height | `F1: Settings` and quit. Context hints drop from the right when the window is narrow. |

Status belongs in the pane title or the footer the app already uses: file name, mode, dirty mark, git badge. The header is not a second toolbar.

Toasts stack above the footer, up to four, and expire after five seconds. Border, glyph, and message use the semantic color. Shift that color toward black or white until the text clears 4.5:1 on the modal background. The terminal uses the same words and the same five-second lifetime.

Body panes use `flex_basis 0` and `min_h(px(0.))` so a long document cannot stretch the window. The document itself is absolutely positioned inside the pane. Pane split sizes are stored in character columns and rows, then converted to pixels with the current face's advance and line height. Dragging a split writes those character counts back. Hovering the boundary shows the column or row resize cursor, and that cursor stays for the whole drag.

Docks, when the app has them:

- The sidebar is visible above 720px, and only when pinned at 720px and below.
- The inspector is visible from 1000px up, and only when pinned below that.
- The title-bar toggles and the chords (`Ctrl+\` for the sidebar, `Ctrl+Alt+\` for the inspector in the reference app) call the same functions. Pick the chords in the shared table and use them in both places.

Density, when the app has it, changes spacing one notch (row, input, button, padding, gaps). Type size, tree indent, and dock widths stay. Comfortable rows are 28px. Compact rows are 24px. An app with one density uses 28px.

The palette and find, when the app has them, are centered sheets with their own widths. They are not the Settings panel. Escape closes them. They do not change dock widths.

---

## Background work

Git, indexing, file watches, and network calls run on `std` threads and return through `mpsc`. The window polls. The GPUI thread does not block on that work.

A busy word goes in the header, using the same four-frame mark as the terminal (`▃ ▅ █ ▅`, 150ms, label `{bar} d_d {bar}`). A finished job paints a toast or a confirm. A quiet index refresh does not.

Writes that the app itself just performed are marked so the watcher does not prompt to reload them. A change from outside prompts when the document is dirty and reloads when it is clean.

---

## Installer

`install.sh` asks which program to install. Uninstall asks the same question and removes only that choice.

| Flag or answer | What it installs or removes |
|---|---|
| `--tui`, `1`, `tui`, `terminal` | The terminal binary and its `.desktop` |
| `--gui`, `2`, `gui`, `window` | The window binary and its `.desktop` |
| `--both`, `3`, `both` | Both binaries and both desktop files |
| (no flag, with a tty) | Prompt. Default is the terminal on install and both on uninstall |
| (no flag, no tty) | Install the terminal app and say so on stderr. Uninstall removes both and says so |
| `INSTALL_GUI=1` with no target flag | Same as `--both` |

More than one of `--tui`, `--gui`, and `--both` is an error. Accept the words above in any case. Lowercase with `tr`. macOS ships bash 3.2, which has no `${var,,}`.

The prompt is written to `/dev/tty` and the answer is read from `/dev/tty`. A real terminal is `{ : <>/dev/tty; } 2>/dev/null`. A `-r` or `-w` test is true even when the process has no controlling terminal. A failed tty redirect inside a captured command must not abort the script (`set -e`). Capture the choice with `target=$(choose_target ...)`, and send every notice to stderr. A notice on stdout becomes part of the captured word. Empty input and EOF use the default.

Uninstall removes the chosen binaries and their `.desktop` files. It leaves the theme YAML, the icon, `$CONFIG_DIR/{app}`, and `~/.config/ldnddev` in place. Say that in the usage text. Pass the same `PREFIX`, `BIN_DIR`, and `XDG_DATA_HOME` used to install.

`data_dir` prefers `$XDG_DATA_HOME` over `PREFIX`. A dry run that sets only `PREFIX` still edits `~/.local/share` when `XDG_DATA_HOME` points there. A test uninstall sets `XDG_DATA_HOME` to a throwaway directory. User installs use `${XDG_DATA_HOME:-~/.local/share}/applications`. A prefix outside `$HOME` uses `$PREFIX/share/applications`.

From a clone, build the terminal with the repo's release profile and the window with the window profile from **Pins**. On Profile B, if `CARGO_TARGET_DIR` is set, the binary is `$CARGO_TARGET_DIR/release/{app}-gui`. Otherwise it is `crates/{app}_gui/target/release/{app}-gui`. The theme file is installed for the terminal and for `--both`. A window-only install does not replace the theme.

Linux release archives, when the app publishes them:

- The terminal is `{arch}-unknown-linux-musl`.
- The window is `{arch}-unknown-linux-gnu`.
- `--both` downloads both archives. `TARGET` is ignored in that mode, with a warning. One triple cannot build both binaries.
- Each archive has a `.sha256`. Refuse a missing or mismatched checksum.

Desktop files, one per binary that exists after install:

- `{app}.desktop` with `Terminal=true`
- `{app}-gui.desktop` with `Terminal=false`, `StartupNotify=true`, and `StartupWMClass=dev.ldnddev.{app}`

Quote `Exec` so a path with spaces survives. Share one icon when the app has one. Refresh a launcher for every binary still present in `BIN_DIR`, so reinstalling the terminal does not drop the window launcher. macOS skips the desktop files.

When stdout is a terminal, the script may print the ldnddev wordmark as half-block art before the prompt or the usage text. Skip it when stdout is not a terminal, so a pipe stays plain text. Draw the mark in bright white on a black background (`\033[40;97m`) and reset after it.

Curl runs the script from a pipe. Guard the entry point so `BASH_SOURCE[0]` is safe under `set -u`:

```bash
if [[ -z "${BASH_SOURCE[0]:-}" || "${BASH_SOURCE[0]:-}" == "$0" ]]; then
  main "$@"
fi
```

`bash -s -- --tui` passes the flag through. `INSTALL_GUI` has to be set on the bash that runs the script.

---

## Tests

Test the window without opening a display. Key maps, footer strings, scroll math, theme packing, settings rows, and paste sanitizing are plain functions. On Stack S, `gpui::test` mounts the shell headless. Mounting does not draw a frame. Assert on a text dump the shell already builds (`visible_text` / `lines`).

Dump lines that lock the sheet on Stack S:

- `settings-panel:right:70` while the sheet is open
- `settings-nav:` listing the pages, and `settings-page:` naming the visible page (`Themes`, `Window`, `Help`, or `Results`)
- `theme-list:flow` on the Themes page, while the names are in the one scroll
- The shortcut sheet text, including `Settings · F1 · F1`, on the Help page
- Density and Vim on the Window page, including after the sheet is closed and opened again

Checks that lock the sheet on either stack:

- F1 opens Settings. F2 opens that same sheet. A second F1 closes it and restores the palette from open.
- Escape reverts. The color editor steps back to the list before the sheet reverts.
- Option rows include their detail and are one hit target. Section boxes add their rule and gap heights to the scroll range.
- Wheel movement changes the scroll offset and leaves the selected row where it was.
- A click on a hex field places the caret after the `Hex: ` prefix.
- Font save merges `[gui]` and keeps `[ai]`.
- Paste of a multi-line string is one insert.
- `gui.json` round-trips in a temp dir and leaves the document file untouched.

A starter document that is missing assets still has validation errors. Assert a clean validation list only when the fixture is clean.

`cargo clippy --all-targets -- -D warnings` is clean on the window and on the terminal workspace. `cargo deny check` is clean for the window. `bash -n install.sh` is clean.

```bash
cargo test -p {app}_core --offline --lib
cargo test -p {app}_gui --offline --bin {app}-gui
cargo test -p {app} --offline --bin {app}
cargo fmt -p {app} -p {app}_gui -p {app}_core
bash -n install.sh
```

Profile B uses `--manifest-path crates/{app}_gui/Cargo.toml` and `CARGO_TARGET_DIR=target/{app}_gui` for the window line.

Open the window on the dev machine before calling a layout done. Exercise the changed path: keyboard, click, wheel, and the matching command-palette item. A unit test does not show whether a border clipped a row or a click landed on padding.

The installed binaries are a separate step. Tests do not replace `~/.local/bin/{app}-gui`. Run `./install.sh --gui` or `./install.sh --both` when the new window should be the one on disk.

---

## What stays in the app

Keep these out of the shared contract:

- The document model and its file format
- The canvas (tree, outline, blueprint, editor fields, note list)
- Which extra commands exist beyond Settings, save, quit, and the docks
- Copy in the shortcut sheet beyond the columns the table already provides
- Preview, export, build, git, and AI, unless that app actually does those jobs

An app-only layout stays in that app's present module and its tests.

---

## Current forks

These are the places the two windows still differ. A new app follows the rule in the section above. Leave a working window on its current side of the fork until that app is ready to move.

| Topic | Contract for a new app | `dd_vault` today | `dd_siteforge` today |
|---|---|---|---|
| GPUI | One stack, pinned with `=` | Stack U, `gpui-unofficial` 1.27.0 | Stack S, `gpui-pre` 0.3.12 and `gpui-component` 0.9.1 |
| Workspace | Profile A when the repo's Rust can compile GPUI. Profile B when the terminal must stay older | Profile B. Own workspace, Rust 1.92, own target dir. Terminal is Rust 1.82 | Profile A. Member, not a default member. Edition 2024. Inherits the workspace release profile |
| Window release | Terminal stays `z` / fat LTO. Window on Stack U uses opt-level 3 / thin LTO | opt-level 3, thin LTO, 16 codegen units | Workspace profile: `z`, fat LTO, 1 codegen unit |
| App id | `dev.ldnddev.{app}` | `dd_vault-gui` | `dev.ldnddev.dd_siteforge`, companion `.preview` |
| First window size | 1280×800 | 1100×720 | 1280×800 |
| Sheet anchor | Terminal: right half of the body. Window: body band, `relative(0.7)`, under the header and above the footer | `top(28) / bottom(28)`, `relative(0.7)`, left nav | `top(header) / bottom(footer)`, `relative(0.7)`, left nav. Terminal sheet is the right half |
| Section metrics | 12px gap, 28px header, 28px theme rows. A title+detail row is 52px | 2px bars, 52px option rows, headings on the body background | 1px rules, 28px theme rows, 48px sheet title, header on `surface.raised`. Density and Vim stay inside the Window page |
| Sheet search | Window: Search on the nav, `/` focuses it, Esc clears the query before the sheet closes. Terminal: no search field | Search, `/`, `[` `]`, and `t` for the save target | Search, `/`, `[` `]`, and `t` for the save target. Pages are Themes, Window, Help |
| Faces | Shared `[gui]` in `ldnddev/config.toml` | Implemented | No face picker. Per-app `gui.json` holds density, Vim, and reduced motion |
| Scroll | One scroll. Reveal on selection change. No nested `max_h` scroller | `ScrollPort`, positive pixels, negative handle | `scroll_to_item(index)` on the Themes page. The lead is 0 |
| Tests | Headless, plus one pass in a real window | Plain functions, clippy `-D warnings`, `cargo deny` in the window crate | `gpui::test` dump lines |
| Install extras | Choice model is required. The rest is per app | No release archive, no icon, no wordmark. macOS bash 3.2 and the curl-pipe guard | musl terminal, gnu window, `.sha256`, shared icon, wordmark on a tty |

---

## Copy checklist

- The terminal program remains the source of behavior. The window calls the same core functions, reads the same config, and writes the same session.
- The window depends on the core, and on the editor, the renderer, and the vendored theme when those crates exist. It does not depend on the terminal crate or on another repository.
- GPUI is pinned with `=`, with the Wayland and X11 platform features, on one of the two stacks.
- `init` and the other disk-only commands print and exit. The window opens only for an interactive launch.
- `app_id`, the desktop `StartupWMClass`, and a companion's `.preview` id follow **Open the window**.
- Theme lookup is local, then global, then built-in, then `apply_omarchy`. Colors come from the packed palette. Faces stay out of the theme YAML.
- Header and footer stay full width while Settings is open. The terminal sheet is the right half of the body. The window sheet is 70% of the body band.
- F1 is one settings sheet with a left nav and one visible page. Themes is first. Help is last. Esc reverts an unsaved theme. `t` toggles the save target. `y` writes the theme.
- Per-app window prefs live in `gui.json`. Shared faces live in `ldnddev/config.toml` `[gui]`.
- Scroll reveals a row only when the selection moves. Theme names are not inside a nested max-height scroller.
- Paste is one insert. Background work is a thread and a channel. A window opened from inside a shell update goes through `cx.defer`.
- The installer asks terminal, window, or both, for install and for uninstall. The window `.desktop` sets `Terminal=false`. Uninstall leaves theme, icon, and config.
- Tests and clippy pass on the terminal workspace and on the window. `cargo deny check` passes. The changed flow has been used in a real window.
