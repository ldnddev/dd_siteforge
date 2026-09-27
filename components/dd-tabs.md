---
component: dd-tabs
version: 1
node_scope: section_item
insert:
  defaults:
    parent_id: "tabs"
    parent_class: "-horizontal"
    aria_label: "Content tabs"
    sal: "fade"
    items:
      - child_title: "Tab 1"
        child_copy: "Panel copy"
fields:
  - id: parent_id
    required: true
    type: string
    default: "tabs"
    maps_to: ".dd-tabs[data-id]"
    notes: "HTML-safe slug used to prefix tab and panel ids."
  - id: parent_class
    required: true
    type: enum
    options: ["-horizontal", "-vertical"]
    default: "-horizontal"
    maps_to: ".dd-tabs class token"
  - id: aria_label
    required: false
    type: string
    default: "Content tabs"
    maps_to: "[role=tablist][aria-label]"
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "fade"
    maps_to: ".dd-tabs[data-sal]"
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
  - id: items
    required: true
    type: array
    min_items: 1
    item_fields:
      - id: child_title
        required: true
        type: string
        maps_to: "[role=tab]"
      - id: child_copy
        required: true
        type: textarea
        accepts: ["markdown", "html"]
        export_transform: "render markdown to html (raw html passthrough)"
        maps_to: "[role=tabpanel]"
edit_ui:
  tab_order:
    - parent_id
    - parent_class
    - aria_label
    - sal
    - items[].child_title
    - items[].child_copy
blueprint:
  label: "dd-tabs"
  show_fields:
    - parent_id
    - parent_class
    - items[].child_title
---

## HTML Template

```html
<div class="dd-tabs [parent_class]" data-id="[parent_id]">
  <div class="dd-tabs__content dd-g">
    <ul class="dd-tabs__menu dd-g" role="tablist" aria-label="[aria_label]">
      <!-- each tab --><li class="dd-tabs__menu-item" role="presentation">
        <button type="button" class="dd-tabs__menu-link [-active]" id="[parent_id]-tab-N" role="tab" aria-selected="true|false" aria-controls="[parent_id]-panel-N" tabindex="0|-1">[child_title]</button>
      </li><!-- endeach -->
    </ul>
    <div class="dd-tabs__items dd-g">
      <!-- each panel --><div class="dd-tabs__item dd-u-1-1 [-active]" role="tabpanel" id="[parent_id]-panel-N" aria-labelledby="[parent_id]-tab-N" tabindex="0" hidden>
        [[child_copy_html]]
      </div><!-- endeach -->
    </div>
  </div>
</div>
```

First tab is active. Inactive panels use the `hidden` attribute. Keyboard arrows/Home/End move and activate tabs.
