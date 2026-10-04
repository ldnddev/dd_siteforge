---
component: dd-your-component
version: 1
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    parent_type: "-default"
    parent_class: "-primary"
    sal: "no-animation"
    parent_title: "Title"
    parent_subtitle: "Subtitle"
    parent_copy: "Copy"
    parent_link_url: "/path"
    parent_link_target: "_self"
    parent_link_label: "Learn More"

fields:
  - id: parent_type
    required: true
    type: enum
    options: ["-default", "-alt"]
    default: "-default"
    maps_to: ".dd-your-component class token"

  - id: parent_class
    required: true
    type: enum
    options: ["-primary", "-secondary"]
    default: "-primary"
    maps_to: ".dd-your-component class token"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-your-component[data-sal]"

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
    required: true
    type: string
    maps_to: ".dd-your-component__title"

  - id: parent_subtitle
    required: true
    type: string
    maps_to: ".dd-your-component__subtitle"

  - id: parent_copy
    required: true
    type: string
    maps_to: ".dd-your-component__copy"
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

  - id: parent_link_url
    required: false
    type: string
    maps_to: ".dd-your-component__link a[href]"

  - id: parent_link_target
    required: false
    type: enum
    options: ["_self", "_blank"]
    default: "_self"
    maps_to: ".dd-your-component__link a[target]"

  - id: parent_link_label
    required: false
    type: string
    maps_to: ".dd-your-component__link a"

edit_ui:
  tab_order:
    - parent_type
    - parent_class
    - sal
    - parent_title
    - parent_subtitle
    - parent_copy
    - parent_link_url
    - parent_link_target
    - parent_link_label

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - parent_type
      - parent_class
      - sal
      - parent_title
      - parent_subtitle
      - parent_copy
      - parent_link_url
      - parent_link_target
      - parent_link_label
    hide_when_editing_component:
      - column.id
      - column.width_class

blueprint:
  label: "dd-your-component"
  show_fields:
    - "parent_title"
---

## HTML Template

```html
<div class="dd-your-component [parent_type] [parent_class]" data-sal="[sal]">
  <div class="dd-your-component__title"><h3>[parent_title]</h3></div>
  <div class="dd-your-component__subtitle"><strong>[parent_subtitle]</strong></div>
  <div class="dd-your-component__copy"><p>[parent_copy]</p></div>
  <div class="dd-your-component__links dd-g">
    <div class="dd-your-component__link">
      <a href="[parent_link_url]" target="[parent_link_target]" class="dd-button -primary">[parent_link_label]</a>
    </div>
  </div>
</div>
```

## Conditional Markup

- render `.dd-your-component__links` only when both `parent_link_url` and `parent_link_label` are non-empty
- when `parent_link_target` is empty, default to `_self`
