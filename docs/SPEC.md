# dd_siteforge — Spec

Living product spec. Replaces the old dated design/plan archive. Update this file when behavior or conventions change.

Companion docs:

- `Architecture.md` — crate map, render/validation rules, key bindings
- `LDNDDEV_TUI_VISUAL_STANDARD.md` — portable TUI theme + shell contract (copy into any new ldnddev TUI)
- `components/dd-*.md` — per-component fields, render rules, validation
- `README.md` — install pointer
- `docs/index.html` — setup + TUI walkthrough with screenshots ([GitHub Pages](https://ldnddev.github.io/dd_siteforge/))

---

## Product

Terminal-UI CMS for authoring framework-native static pages. Single Rust binary. Author edits a typed site tree in the TUI, exports HTML, hosts anywhere static.

Target: small marketing sites (roughly 5–20 pages, one editor). No multi-user, no live CMS, no database.

Workflow:

1. `init-site` → starter `site.json`, build kit (`source/`, Grunt, Lando/DDEV), and `source/templates/`
2. Images in `./source/images/` next to the JSON
3. `dd_siteforge` in a folder with `site.json` opens the TUI (or a recents/path picker). Last page and tree row restore from `~/.config/ldnddev/dd_siteforge/session.json`
4. TUI edits pages/components/head. Autosave every 2s; `s` writes a `.backup`
5. `Shift+B` runs `lando grunt build` when `.lando.yml` exists, else `npx grunt build`
6. Export: HTML + Grunt `web/assets` + images + sitemap/search-index/robots/404
7. `Shift+P` / `serve` starts a local HTTP server so relative `assets/` paths resolve; preview HTML live-reloads after save/export/build

Current crate version: see `Cargo.toml`.

---

## Shipped surface

### CLI

Bare `dd_siteforge` (or `dd_siteforge site.json`) opens the TUI. `init-site` (`--name`) · `init-templates` (`--force`, `--name`) · `init-scaffold` (`--force`, `--global`, `--name`) · `tui` · `validate-site` · `export-html` · `serve` · `show-site`

### Content model

`Site` → always-present `header` / `footer` → `pages[]` → per-page `head` + `nodes[]` (`dd-hero` or `dd-section` with columns of components). Optional `export_dir`, `base_url`, `lang`, `pretty_urls`.

Page head: `title` is the TUI page label. `meta_title` is the HTML `<title>`; empty falls back to `title`. Slug is edited on the same form and may contain `/` for folders (`blog`, `blog/entry`). Home is always `index.html`. With **Pretty URLs** on (Site settings), other pages write `{slug}/index.html`; off, `{slug}.html`. Do not use `index` as a folder name — slug `blog` is the blog index.

New fields on `Site` / `Page` take `#[serde(default)]` so legacy JSON still loads.

Animation attributes are SAL (`data-sal`, optional `data-sal-duration` / `data-sal-delay`). Style includes `no-animation` (omits all three attributes). Duration is 200–2000 ms in 50 ms steps (CSS default 400, omitted from HTML when unset or 400). Delay is 0–1000 ms in 50 ms steps; collection items (card, alternating, milestones) add `100 × index` on top, capped at 1000. JSON still accepts the old `parent_data_aos` alias. Slider and modal have no SAL fields.

Buttons are `DdLink` (`url`, `label`, `target` `_self`/`_blank`, `style` `-primary`/`-secondary`/`-tertiary`/`-ghost`). Hero has `links` (max 2); CTA, alternating items, and slider items have `links` (max 4). Empty list means no buttons. Legacy hero `link_1_*` / `link_2_*` and CTA/slider single-link fields still load when `links` is empty (second hero link defaults to `-ghost`). Card and milestones items keep one optional triple plus `child_link_style`. Image wraps the picture (URL + target only).

Section visual options: `section_class` (width), optional `bg` (`-bg-muted`), `padding` (`-no-padding`), `custom_css`, `aria_label`, and SAL (default `no-animation` so legacy JSON does not animate). Builder `id` is emitted as the HTML `id`. Accessible name is title, else `aria_label`, else `"Content section"`.

Hero overlay / copy position: optional overlay (`-overlay-light` / `-overlay-dark`, omitted when none), copy position (`-left` / `-center` / `-right`) as a separate field from width `parent_class`. Optional HTML-safe `id` unique among heroes and sections on the page (emitted only when set). ARIA label defaults to `"Introduction"`. Missing JSON overlay/id/aria stay omitted; missing copy position becomes `-left` on next save.

Site options: header root has CTA label/URL (empty URL hides the button; default label `"Contact"`) and a plain-text alert banner (`.dd-header__banner`, separate from the optional header `dd-alert`). Footer root has blurb, copyright override (empty falls back to `© {year} {site.name}`), and LinkedIn / X / GitHub URLs. Site settings holds header and body GTM snippets; export extracts a `GTM-XXXX` id and emits canonical googletagmanager.com script/noscript. Unrecognized snippets are omitted from HTML and fail validation.

Spacer, tabs, timeline, and data table are section components. Spacer is size (`-sm`…`-xxxl`) plus optional divider; no SAL; `aria-hidden="true"`. Tabs have a slug `parent_id`, orientation (`-horizontal` / `-vertical`), ARIA label (default `"Content tabs"`), SAL on the root, and items of label + markdown panel (min 1). Timeline has ARIA label (default `"Timeline"`), SAL stagger on events, and items of year, optional datetime, title, heading level 2–6 (default 3), markdown copy, and optional image. Datetime is required when year is not YYYY / YYYY-MM / YYYY-MM-DD. Data table has a required caption, optional dense spacing, optional scroll label (default `"{caption}, scrollable"`), optional empty message, 1–5 columns (label, align `start`/`center`/`end`, sortable), and rows of cells (`text` or `badge` with `-critical`/`-warning`/`-info`/`-pass`). First column/cell is the row header. No SAL. Sort keys are derived from column labels. Scroll `tabindex`/`role`/`aria-label` are JS-owned.

Media on hero, banner, alternating items, and slider items is `Media`: `none` / `image` / `oembed` / `local-video`. Image requires URL + alt. oEmbed stores a YouTube or Vimeo URL and exports a nocookie/player iframe. Local video needs large MP4 + accessible name; small MP4, poster, loop, and autoplay are optional. Autoplay is muted and stripped when `prefers-reduced-motion: reduce`. Legacy `parent_image_url`/`alt` (and `child_image_*`) load as `image` when `media` is absent. CTA, card, filmstrip, blockquote, and `dd-image` stay image-only. Hero image class and breakpoint URLs show when kind is `image`. Ctrl+P on image/poster fields lists images; on `mp4` fields lists `.mp4` in `./source/images/`.

### TUI

Fixed 3-line header + body + 1-line adaptive footer per the visual standard.

Panes: `[1]` Regions · `[2]` Pages · `[3]` Layout · `[4]` Details (focusable; click-to-select, ascii maps).

Toasts for success / info / warning. Modals for errors and forms.

F1 Help (wrap + scroll). F2 Theme (source, status, color samples; same chrome as F1). F3 Validate. F4 page health. `Shift+E` Export. `Shift+P` Preview (live-reload on the local server). `Shift+B` Lando/npx grunt. `?` / `Ctrl+F` find. `:` / `Ctrl+K` command palette. `s` Save. `/` insert. `.` repeat last insert. `Ctrl+R` redo. `Ctrl+Q` quit (confirm if dirty). Selecting `[HEAD]` prepends a page-health checklist in Details.

Pages panel: add / delete / undo / reorder / rename. Layout: nav, expand, edit, copy (`y`) / paste (`p`), columns. Column ids are `{section-id}-column-N` (the section id is prefixed on save) so they stay unique on the page; renaming a column keeps its components.

Edit forms: Tab between fields, click-to-focus, click in a text field or textarea to place the caret, drag or Shift+click to select, double-click to select a word, mouse wheel, `Ctrl+P` image or page picker on URL fields. Image, poster, and mp4 URL fields also show a Browse button that opens the same file picker. `Shift+arrows` (plus Shift+Home/End and Shift+Ctrl+arrows) grow a range; `Ctrl+A` selects the field; unshifted Left/Right jump to the range start/end; typing, paste, Backspace, and Delete replace the range. Terminal paste inserts a clipboard dump in one shot. `Ctrl+Z` undoes the last text edit. `Ctrl+←` / `Ctrl+→` / `Ctrl+Backspace` / `Delete` edit by word or forward-delete. Textareas wrap on spaces. `Ctrl+S` also re-exports when a preview server is already running; `Shift+P` again re-exports without opening a new browser tab.

### Export + assets

- Copy textareas with Expand (`dd-hero`, `dd-rich_text`, `dd-cta`, `dd-alert`, `dd-modal`, `dd-blockquote`, plus card / accordion / alternating / milestones / slider / tabs / timeline item copy) render CommonMark to HTML at export (headings, lists, thematic breaks, code, tables, strikethrough, raw HTML passthrough). JSON-LD `text` keeps the authored source.
- Handlebars load order (later wins): baked-in crate `templates/*.hbs`, then the crate `templates/` directory when this binary was built from a checkout that still exists, then `<site>/templates/`, then `<site>/source/templates/`. A `source/templates` file that is still an exact copy of the baked-in template yields to the live crate file. `init-templates` copies from the crate `templates/` tree when present (`--force` overwrites). Export never writes templates.
- Build kit (Gruntfile, package.json, `.lando.yml`, `.ddev/`, `source/` except author images and templates) is embedded in the binary. `init-site` copies it once (skip existing). Optional house overlay: `~/.config/ldnddev/dd_siteforge/` (dump with `init-scaffold --global`). Re-seed a site with `init-scaffold --force`.
- `init-site` asks for a project name (or `--name` / folder default when stdin is not a TTY) and stamps that slug into Lando, DDEV, and `package.json`.
- CSS/JS come from Grunt (`source/{js,scss}` → `web/assets`). `Shift+B` in the TUI runs `lando grunt build` when `.lando.yml` is present (Lando owns Node); otherwise `npx grunt build`. If Lando is configured but missing or not started, the TUI shows the error instead of falling back to host Node. Host `npm install && npx grunt build` remains valid from a shell. DDEV is still an optional wrapper.
- Export copies Grunt `web/assets/{css,js,webfonts,favicon,vendors}` unless dest is already that tree (does not clobber a local grunt build). Fills missing webfonts/favicon from `source/`. Copies `source/images/` → `<out>/assets/images/`.
- Export also writes `search-index.json` next to `sitemap.xml` for client-side JS search. Schema: `{ "v": 1, "pages": [ { "url", "path", "title", "description", "body", "headings", "image"? } ] }`. Pages with `noindex` robots are omitted (same as sitemap). Generated `404.html` is omitted because it is not in `site.pages`. Header and footer copy are omitted so chrome labels do not match every page. `url` is the export-root `page_href` (`index.html`, `{slug}.html`, or `{slug}/index.html`). `path` is the display path (`/`, `/{slug}.html`, or `/{slug}/`). `title` is `head.title`. Description is `meta_description`, then `og_description`, then the first ~160 characters of body. Body is identifying page copy (hero/section/component titles, subtitles, copy, link labels, alts, collection items, data-table cells) with markdown stripped, whitespace collapsed, capped at 12k characters. `headings` are unique section/hero/card/accordion/tab titles (case-insensitive). `image` is `og_image` when set. Every page stamps `data-search-index="search-index.json"` on `<html>`; nested pages prefix it with `../` like other site-relative URLs. Fetch that path from the current document and resolve result links as index-directory + `url`.

### Validation

`validate_site` — structural. `validate_site_with_root` — also missing local images. CLI export/serve and TUI F3/export/preview all gate on this.

---

## Non-goals

- Multi-user, auth, or a server-side CMS
- Remote image CDN beyond pasting an external URL
- Mixing template seed with the Grunt pipeline
- Inventing one-off theme tokens (promote them in the visual standard first)

---

## Conventions

### Branch + commit

Feature work on `feat/<short-name>` off `master`. Commits use plain prefixes: `tui:`, `model:`, `validate:`, `docs:`, `test:`. Tags: annotated `vMAJOR.MINOR.PATCH`. Fast-forward merges. Do not re-tag.

### Tests

`#[cfg(test)] mod tests` at the bottom of the module. Drive TUI via in-tree `send_key`, not by poking state when a key path exists. `cargo test -q`.

### New component

1. Spec in `components/dd-*.md` (copy a `components/*-template.md` scaffold)
2. Types in `src/model.rs`
3. Renderer in `src/renderer.rs` + `templates/dd-*.hbs`
4. FormEdit in `src/tui/editform/`
5. Route in `src/tui/cursor.rs`

### New modal

Four-point plumbing: enum variant + render dispatch + event dispatch + `Modal::variant_name` arm. Multi-field forms go through `render_edit_modal_unified` or `render_form_edit_modal`. Single prompts share `render_single_input_modal`. Render fns are `&self`; values the event loop needs from draw go through `RefCell` or pre-publish into `&mut self`.

### Theme

Always `self.theme.*`. Labels: `text_labels` → `text_active_focus` when focused. Input borders/text/cursor from the `input_*` + `cursor` tokens. Folders/files/links in pickers. Modal section headers: `modal_header` (bold).

### UX prefs

- Footer is a 1-line adaptive key bar. Always starts with `F1:Help` then `F2:Theme`. No long status text.
- All scrollable surfaces: mouse wheel + keyboard.
- Path display: strip leading `./` and trailing `/`.
- Browser launch pins stdio to `/dev/null` so raw-mode TUI stays intact.

### Local files — never commit

`site.json`, `site.json.backup`, `web/`, `source/images/*` (keep `.gitkeep`), `.kilo/`.

### Ship

Smoke test → fast-forward merge → push → annotated tag matching `Cargo.toml` version → push tag. No PR unless asked.

---

## Anti-patterns

- Features that were not requested
- Proactive `cargo fmt` / `cargo fix`
- Bypassing git hooks
- Back-compat shims for paths with no old consumers
- Docstrings on every function (one-liner only when the *why* is non-obvious)
- Overwriting existing user files on template seed without `--force`
