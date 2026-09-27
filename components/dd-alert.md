---
component: dd-alert
version: 1
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    parent_type: "-default"
    parent_class: "-primary"
    sal: "fade"
    parent_title: "Title"
    parent_copy: "Copy"

fields:
  - id: parent_type
    required: true
    type: enum
    options: ["-default", "-info -minor", "-warning -moderate -serious", "-error -critical", "-success"]
    default: "-default"
    maps_to: ".dd-alert class token"

  - id: parent_class
    required: true
    type: enum
    options: ["-default", "-compact"]
    default: "-default"
    maps_to: ".dd-alert class token"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "fade"
    maps_to: ".dd-alert[data-sal]"

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


  - id: parent_title
    required: false
    type: string
    maps_to: ".dd-alert__title"

  - id: parent_copy
    required: true
    type: string
    accepts: ["markdown", "html"]
    export_transform: "render markdown to html (raw html passthrough)"
    maps_to: ".dd-alert__copy"
    ui:
      control: textarea
      rows: 3
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
    - parent_type
    - parent_class
    - sal
    - parent_title
    - parent_copy

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - parent_type
      - parent_class
      - sal
      - parent_title
      - parent_copy
    hide_when_editing_component:
      - column.id
      - column.width_class

blueprint:
  label: "dd-alert"
  show_fields:
    - "parent_title"
---

## HTML Template

```html
<div class="dd-alert [parent_type] [parent_class]" role="alert" data-sal="[sal]">
  <div class="dd-alert__content dd-g">
    <div class="dd-u-1-1">
      <div class="l-box">
        <div class="dd-alert__title">
          [parent_title]
        </div>
        <div class="dd-alert__copy">
          [[parent_copy_html]]
        </div>
      </div>
    </div>
  </div>
</div>
```

## Conditional Markup

- render `.dd-alert__title` only when `parent_title` is non-empty
