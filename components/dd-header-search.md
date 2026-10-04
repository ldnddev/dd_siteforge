---
component: dd-header-search
version: 2
node_scope: header_item   # header-only chrome component; cannot be used in page sections

insert:
  defaults:
    sal: "no-animation"

fields:
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-header__search-icon[data-sal]"

  - id: sal_duration
    required: false
    type: enum
    options: [200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 950, 1000, 1050, 1100, 1150, 1200, 1250, 1300, 1350, 1400, 1450, 1500, 1550, 1600, 1650, 1700, 1750, 1800, 1850, 1900, 1950, 2000]
    default: 400
    visible_when: "sal != no-animation"
    maps_to: "[data-sal-duration]"
    notes: "Omitted from HTML when unset or 400 (CSS default)."

  - id: sal_delay
    required: false
    type: enum
    options: [0, 50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850, 900, 950, 1000]
    default: 0
    visible_when: "sal != no-animation"
    maps_to: "[data-sal-delay]"
    notes: "Collection items (card, alternating, milestones) add 100×index, cap 1000."


edit_ui:
  tab_order:
    - sal

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - sal

blueprint:
  label: "dd-header-search"
  show_fields:
    - sal
---

## HTML Template

```html
<div class="dd-header__search-icon -y-center -x-center" data-sal="[sal]">
  <button class="dd-search__toggle fa-regular fa-magnifying-glass" type="button">
    <span class="visually-hidden">Search</span>
  </button>
</div>
```

## Conditional Markup

- always renders when present in a header item's `components[]`
- the actual search dropdown panel (`<div class="dd-search">…</div>`) is still hardcoded in `dd-header.md` chrome; this component only renders the toggle button
- layout width comes from the parent column's `width_class`

## Validation Rules

- `sal` required; must be one of the enum options
- this component is only valid inside a `dd-section` that is itself a child of `site.header.sections[]`; placing it in a page-level section or in the footer must fail validation
