---
component: dd-headline
version: 1
node_scope: section_item

insert:
  defaults:
    text: "Headline"
    heading_level: "h2"
    custom_css: ""
    sal: "no-animation"

fields:
  - id: text
    required: true
    type: string
    default: "Headline"
    maps_to: "heading text content (plain text, escaped)"

  - id: heading_level
    required: true
    type: enum
    options: ["h2", "h3", "h4", "h5", "h6"]
    default: "h2"
    maps_to: "heading element name"

  - id: custom_css
    required: false
    type: string
    default: ""
    maps_to: "extra class tokens on the heading"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: "heading[data-sal]"

  - id: sal_duration
    required: false
    type: enum
    options: [200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 950, 1000, 1050, 1100, 1150, 1200, 1250, 1300, 1350, 1400, 1450, 1500, 1550, 1600, 1650, 1700, 1750, 1800, 1850, 1900, 1950, 2000]
    default: 400
    visible_when: "sal != no-animation"
    maps_to: "[data-sal-duration]"

  - id: sal_delay
    required: false
    type: enum
    options: [0, 50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 950, 1000]
    default: 0
    visible_when: "sal != no-animation"
    maps_to: "[data-sal-delay]"

edit_ui:
  tab_order:
    - text
    - heading_level
    - custom_css
    - sal
    - sal_duration
    - sal_delay

blueprint:
  label: "dd-headline"
  show_fields:
    - text
    - heading_level
---

## HTML Template

The heading element is the component root. There is no wrapper. Text is plain
and escaped. `h1` is not offered; the page hero owns the document title.

```html
<h2 id="[slug]" class="dd-headline [custom_css]" [data-sal]>[text]</h2>
```

`id` is the kebab-case form of `text` (`Five Seconds` → `five-seconds`) so a
table of contents can link to `#five-seconds`. A second heading with the same
slug, or a slug already used by a section, column, hero, tab, or modal on the
page, becomes `five-seconds-2`. Text that slugifies to nothing uses `headline`.
`[custom_css]` is omitted when empty. SAL attributes are omitted when style
is `no-animation`. Heading level `h3`–`h6` replaces the `h2` tag.

## Validation Rules

- valid in page, header, and footer section columns
- `text` required and non-empty
- `heading_level` is `h2`–`h6`, default `h2`
- `custom_css` optional
- SAL uses the shared rules (duration 200–2000 step 50, delay 0–1000 step 50)
