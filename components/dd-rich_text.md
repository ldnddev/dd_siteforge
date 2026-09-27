---
component: dd-rich_text
version: 2
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    parent_class: ""
    sal: "fade"
    parent_copy: "Copy"

fields:
  - id: parent_class
    required: false
    type: string
    default: ""
    maps_to: ".dd-rich_text class token (append to base class)"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "fade"
    maps_to: ".dd-rich_text[data-sal]"

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


  - id: parent_copy
    required: true
    type: string
    maps_to: ".dd-rich_text__copy inner HTML (rendered from Markdown/HTML)"
    ui:
      control: textarea
      rows: 5
      multiline: true
      keyboard:
        enter: "insert newline"
        ctrl_s: "save"
        up_down: "move cursor line"
        left_right: "move cursor character"
      mouse:
        wheel: "scroll lines"

edit_ui:
  tab_order:
    - parent_class
    - sal
    - parent_copy

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - parent_class
      - sal
      - parent_copy
    hide_when_editing_component:
      - column.id
      - column.width_class

blueprint:
  label: "dd-rich_text"
  show_fields:
    - "parent_copy"
---

## HTML Template

`parent_copy` is interpreted as Markdown/HTML at render time (same conversion
used by `dd-hero.copy`) and injected unescaped into the inner container.

```html
<div class="dd-rich_text [parent_class]" data-sal="[sal]">
  <div class="dd-rich_text__copy">[[parent_copy_html]]</div>
</div>
```

`[[parent_copy_html]]` denotes unescaped output of the Markdown-to-HTML
conversion of `parent_copy`.

## Conditional Markup

- `[parent_class]` class token is appended to `.dd-rich_text` only when `parent_class` is non-empty
- `parent_copy` is CommonMark (plus strikethrough, tables, and task lists): headings (`#`–`######`, setext `=`/`-`), lists (`-`/`*`/`+` and `1.`), thematic breaks (`---`/`***`), blockquotes, fenced code, paragraphs, inline (`**bold**`, `*italic*`, `` `code` ``, `~~strike~~`, `[text](url)`), and raw HTML; conversion is the same one used by every Expand textarea (`dd-hero.copy`, `dd-cta`, `dd-alert`, `dd-modal`, `dd-blockquote`, and item `child_copy` on card / accordion / alternating / milestones / slider)

## Validation Rules

- `parent_copy` required and non-empty (after trimming whitespace)
- `parent_class` optional; when provided, must be a non-empty string (no enum constraint — any CSS class token)
- `sal` required; must be one of the enum options
