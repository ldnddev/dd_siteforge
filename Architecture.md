# Architecture

Terminal-UI CMS for authoring framework-native static pages. Built in Rust on `ratatui` (rendering), `crossterm` (terminal events), `serde`/`serde_json` (state persistence), and `handlebars` (HTML export). Single-binary, no server, no database.

Living product spec: `docs/SPEC.md`. Visual contract: `LDNDDEV_TUI_VISUAL_STANDARD.md`. Component fields: `components/dd-*.md`.

## Crate Layout

```
src/
  main.rs          CLI entry: init-site / init-templates / init-scaffold / show-site / validate-site / export-html / tui
  model.rs         Site → Page → PageNode → SectionComponent typed tree (serde)
  storage.rs       JSON load/save
  validate.rs      validate_site() + validate_site_with_root() (missing-image)
  renderer.rs      typed-model → HTML via handlebars templates
  templates.rs     load bundled, live crate templates/, site templates/, source/templates; seed from live crate tree when present
  scaffold.rs      embed Grunt/source/Lando/DDEV; seed on init; optional ~/.config overlay
  tui/mod.rs            App shell (struct, run loop, save/autosave)
  tui/draw.rs           header / sidebar / details / footer frame
  tui/events.rs         keyboard, mouse, Pages-panel dispatch
  tui/modals/           Modal enum + paint / prompts / pickers / form_edit / events / export / toasts
  tui/tree/             layout tree (build, nav, expand, open, edit, items, columns)
  tui/details/          Details panel (ascii maps, labels, click-to-select)
  tui/theme.rs          TUI theme load
  tui/help.rs           F1/F2 modal text
  tui/cursor.rs         component → form-state mapping
  tui/editform/         FormEdit types + block / collection / layout form values
  tui/component_kind.rs insert-picker kinds
  tui/form_textarea.rs  FormEdit textarea layout
  tui/util.rs           small TUI helpers
source/            framework source (js, scss, favicon, webfonts; images are local)
Gruntfile.js       builds source/{js,scss} → web/assets
package.json       npm run build / dev
.lando.yml         optional Lando (host npm is the contract)
.ddev/             optional DDEV
```


## Content Hierarchy

```
Site
├── header (DdHeader)         always present
├── footer (DdFooter)         always present
├── pages: Vec<Page>
│   ├── head (DdHead)         page label, slug, meta title/description, SEO
│   └── nodes: Vec<PageNode>  ordered top-level blocks
│       ├── Hero(DdHero)      standalone, no wrapper
│       └── Section(DdSection)
│           └── columns → components
├── export_dir: Option<String>
├── base_url: Option<String>  origin for canonical / OG / sitemap
└── lang: String              <html lang>, default "en"
```

## Components

**Top-level (Page node):** `dd-hero`, `dd-section`.

**Section components:** `dd-alert`, `dd-banner`, `dd-blockquote`, `dd-card`, `dd-cta`, `dd-filmstrip`, `dd-image`, `dd-milestones`, `dd-modal`, `dd-rich_text`, `dd-slider`, `dd-alternating`, `dd-accordion`, `dd-navigation`, `dd-spacer`, `dd-tabs`, `dd-timeline`.

**Header / Footer slots:** same component set as section components plus `dd-header-search`, `dd-header-menu`.

Each component spec lives in `components/dd-*.md` (single source of truth for fields, render rules, validation).

## Renderer

- Iterates `site.pages` in order. Home (`index`) is always `index.html`. Nested slugs (`blog/entry`) create folders. With site `pretty_urls`, non-home pages write `{slug}/index.html` (so `blog` → `blog/index.html`, `blog/entry` → `blog/entry/index.html`); otherwise `{slug}.html`. Nested pages prefix `href`/`src` with `../` so `assets/` and page links still resolve.
- `dd-hero` / `dd-section` / each section component has a dedicated `render_*` fn in `src/renderer.rs`.
- Special cases:
  - `dd-accordion` emits FAQ JSON-LD only when `parent_type == -faq`.
  - `dd-blockquote` emits Quotation JSON-LD.
  - `dd-modal` derives `parent_modal_id` from `parent_title` (HTML-id-safe).
  - `dd-slider` derives `parent_uid` from `parent_title`; `uid-<random6>` fallback.
  - Copy textareas with the Expand control (`dd-hero.copy`, `dd-rich_text.parent_copy`, `dd-cta.parent_copy`, `dd-alert.parent_copy`, `dd-modal.parent_copy`, `dd-blockquote.parent_copy`, and `child_copy` on card / accordion / alternating / milestones / slider / tabs / timeline items) accept CommonMark (headings, lists, thematic breaks, code, tables, strikethrough) or raw HTML, converted at export. JSON-LD `text` fields keep the authored source.
- SAL: `sal` style (`no-animation` omits attributes), optional `sal_duration` / `sal_delay`. Collection stagger is author delay + `100 × index`, cap 1000.
- Links: `DdLink` (`url`, `label`, `target`, `style` `-primary`/`-secondary`/`-tertiary`/`-ghost`). Hero max 2; CTA / alternating items / slider items max 4. Card and milestones keep one optional link plus style.
- Media: `Media` enum on hero, banner, alternating items, slider items (`none` / `image` / `oembed` / `local-video`). Shared `_media.hbs` partial. CTA/card/filmstrip/blockquote/`dd-image` stay image-only.
- Section visual options: width (`section_class`), optional `-bg-muted` / `-no-padding` / extra CSS / ARIA label / SAL. HTML `id` on `<section>`.
- Hero overlay / copy position: optional `-overlay-light` / `-overlay-dark` and `-left` / `-center` / `-right` on `.dd-hero` next to width. Optional HTML `id` (emitted when set). ARIA label defaults to `"Introduction"`.
- Spacer: size tokens `-sm`…`-xxxl`, optional `-divider`, `aria-hidden="true"`. No SAL.
- Tabs: `parent_id`, orientation `-horizontal`/`-vertical`, ARIA label default `"Content tabs"`, SAL on the root, items of label + markdown panel. Buttons with APG tablist roles; first tab active.
- Timeline: optional ARIA label default `"Timeline"`, SAL stagger on items, events with year, optional datetime, title, heading 2–6 (default 3), markdown copy, optional image. `<ol>` of events. Datetime required when year is not YYYY / YYYY-MM / YYYY-MM-DD.
- Data table: required caption, optional `-dense`, optional scroll label (default `"{caption}, scrollable"`), optional empty message, 1–5 columns (label / align / sortable), rows of text or `dd-badge` cells. First cell is `<th scope="row">`. No SAL. `sort_key` is derived from the column label slug. JS owns overflow `tabindex`/`role`/`aria-label`.
- Site options: header CTA + banner on `DdHeader`; footer blurb / copyright / socials on `DdFooter`; GTM snippets on `Site`. GTM export extracts `GTM-XXXX` and emits canonical googletagmanager.com markup.
- Static export: `crate::export::export_site(&site, &out, site_root)`. Writes page HTML (see slug rules above), copies Grunt `web/assets/{css,js,webfonts,favicon,vendors}` when the dest is not already `web/` (does not clobber a local `grunt build`), fills missing webfonts/favicon from `source/`, copies `<site_dir>/source/images/` → `<out>/assets/images/`, plus `sitemap.xml`, `robots.txt`, and `404.html` when no author 404 page exists.
- Asset and page hrefs are relative to the export root (`assets/css/style.min.css`, `contact.html` or `contact/index.html`). Nested pages prepend `../`. `Shift+P` / `serve` start a local HTTP server so those paths resolve; a directory URL serves `index.html`.

## Validation

`validate_site(&Site) → Vec<String>`: structural checks (unique slugs and output paths, nested slug segments, paired link fields, required fields per component, etc.).

`validate_site_with_root(&Site, Option<&Path>)`: superset that also resolves every `assets/images/*` URL against `<root>/source/images/` (pages, header, footer, `og_image`) and reports missing files as `Missing local image: …`. CLI `validate-site` / `export-html` / `serve` and the TUI F3/export/preview gates all use this when a site path is known.

## TUI Loop

`fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>)`:

```
draw once
loop:
  poll a running Lando/npx grunt build (toast when it finishes)
  wait until an event, autosave due, toast expiry, or ~200ms while a build is running
  drain every pending event, then mark_dirty_if_changed only if a handler mutated the site
  prune expired toasts
  tick_autosave(now)
  draw only if the UI changed
```

Key release, mouse move/up, and focus events skip the following frame. The 200ms build poll does not paint unless a message arrived. Toast expiry and a successful autosave (header `*`) still force a redraw. `Event::Resize` always redraws so mouse hit-tests stay current.

Bracketed paste is enabled for the session. `Event::Paste` inserts the clipboard as one string in forms and prompts. Navigation keys skip the JSON dirty snapshot.

### Key bindings (global)

| Key | Action |
|---|---|
| `F1` | Help (scrollable) |
| `F2` | Theme info modal (source + status + color details; same layout as F1) |
| `F3` | Validate site → modal on errors, success toast otherwise |
| `F4` | Page health for the current page (SEO, alt text, headings, internal links) |
| `Shift+E` | Export site (validate gate → render → copy source/images/) |
| `Shift+P` | Preview current page (validate → export → local HTTP server → browser). A second press re-exports and reuses the tab. Local HTML includes a live-reload snippet |
| `Shift+B` | Build CSS/JS: `lando grunt build` when `.lando.yml` exists, else `npx grunt build` |
| `?` / `Ctrl+F` | Find in the site (page titles, copy, field values) and jump the tree |
| `:` / `Ctrl+K` | Command palette |
| `s` | Save (writes `<path>` + `<path>.backup`) |
| `/` | Insert component fuzzy picker |
| `.` | Repeat last insert |
| `Tab` / `Shift+Tab` | Next/prev page |
| `1` / `2` / `3` | Sidebar focus: Regions / Pages / Layout |
| `Ctrl+R` | Redo last tree edit |
| `Ctrl+Q` | Quit (confirms if unsaved) |

### Pages panel (`[2] Pages`)

`Shift+A` add (template picker) · `Shift+X` delete (confirm + session trash) · `u` undo delete · `Shift+J/K` reorder · `r` rename.

### Layout panel (`[3]`)

`Up/Down` or `j/k` move row · `g`/`G` first/last · `h`/`l` collapse/expand · `Space` toggle expand · `Enter` edit row · `d` delete selected grain · `y` copy · `p` paste after · `u` undo · `Ctrl+R` redo · `.` repeat last insert · `J/K` move selected grain down/up · `C/V` add/remove column · `c/v` prev/next column · `r/f` edit column id/width-class. Selecting `[HEAD]` prepends a page-health checklist in Details.

### Edit modal

`Tab` / `Up/Down` navigate fields · `Left/Right` cycle enum values · `Ctrl+←` / `Ctrl+→` word jump · `Ctrl+Backspace` delete word · `Delete` forward-delete · `Ctrl+Z` undo last text edit · terminal paste inserts in one shot · `Ctrl+S` save (refreshes a running preview) · `Esc` cancel · `Ctrl+P` (in URL field) opens image picker (image fields) or page picker (link fields). Click any input box to focus it. Mouse wheel scrolls the field list. Textareas wrap on word boundaries.

### Image / Page pickers

`↑/↓` move · `←` parent dir (image only) · `→`/`Enter` descend or pick · type to filter · `Esc` cancel.

## Theme + Visual Shell

Theme load is strict (LDNDDEV_TUI_VISUAL_STANDARD.md):
1. `./dd_siteforge_theme.yml` (local)
2. `~/.config/ldnddev/dd_siteforge_theme.yml` (global)
3. Built-in defaults

Every theme file must contain `version: 1` at the top level (validated on load; bad/missing version falls back with a Warning toast).

`header_quotes` (optional top-level list) overrides the 5 built-in rotating header taglines (chosen once at App::new using time ^ pid).

The TUI now exposes `theme.app_shell` and `theme.active_border` (Style) for the standard header/footer.
`theme.modal_header` colors inner section headers (e.g. cards in F1 help modal).
F2 opens the Theme info modal (source, load status, sampled color tokens with hex) using identical chrome and scroll mechanics to the F1 help modal.
Header title is `dd_siteforge` (product name only). Version lives in F2 Theme. Footer always starts with `F1:Help` then `F2:Theme`.

See `src/tui/draw.rs` and `src/tui/theme.rs` (AppTheme::load, choose_header_copy) for the concrete implementation.
The Details panel title now includes current page context ("Details — 03: Home").
Former persistent status messages are delivered as toasts.

## Storage + Autosave

- JSON via serde, pretty-printed.
- Dirty-detection compares a serialized snapshot of the site against `last_saved_json` after each event.
- Autosave: 2s debounce → write to current path. Skipped when no path is set.
- Manual `s`: writes `<path>` AND a byte-identical `<path>.backup` (last-known-good checkpoint).
- On load: if `<path>.backup` exists and differs from `<path>`, surface an Info toast.

## Testing

`cargo test -q` — 102 TUI tests; 152 crate-wide across model, storage, validate, and TUI integration paths. Integration tests drive the App via synthesized key events using the in-tree `send_key` helper.
