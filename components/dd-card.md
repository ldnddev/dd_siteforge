---
component: dd-card
version: 1
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    # parent fields
    parent_type: "-default"
    sal: "no-animation"
    parent_width: "dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24"

    # required children collection
    items:
      - child_image_url: "https://dummyimage.com/720x720/000/fff"
        child_image_alt: "Image alt text"
        child_title: "Title"
        child_subtitle: "Subtitle"
        child_copy: "Copy"
        child_link_url: "/path"
        child_link_target: "_self"
        child_link_label: "Learn More"

fields:
  # ---------------------------
  # parent fields
  # ---------------------------
  - id: parent_type
    required: true
    type: enum
    options: ["-default", "-horizontal"]
    default: "-default"
    maps_to: ".dd-card class token"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-card[data-sal] OR child[data-sal]"

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


  - id: parent_width
    required: true
    type: string
    default: "dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24"
    maps_to: ".dd-card__item class token"

  # ---------------------------
  # child items[] fields
  # ---------------------------
  - id: items
    required: true
    type: array
    min_items: 1
    item_fields:
      - id: child_image_url
        required: true
        type: string
        maps_to: ".dd-card__image img[src]"

      - id: child_image_alt
        required: true
        type: string
        maps_to: ".dd-card__image img[alt]"

      - id: child_title
        required: true
        type: string
        maps_to: ".dd-card__title"

      - id: child_subtitle
        required: true
        type: string
        maps_to: ".dd-card__subtitle"

      - id: child_copy
        required: true
        type: string
        accepts: ["markdown", "html"]
        export_transform: "render markdown to html (raw html passthrough)"
        maps_to: ".dd-card__copy"
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

      - id: child_link_url
        required: false
        type: string
        maps_to: ".dd-card__link a[href]"

      - id: child_link_target
        required: false
        type: enum
        options: ["_self", "_blank"]
        default: "_self"
        maps_to: ".dd-card__link a[target]"

      - id: child_link_label
        required: false
        type: string
        maps_to: ".dd-card__link a"

      - id: child_link_style
        required: false
        type: enum
        options: ["-primary", "-secondary", "-tertiary", "-ghost"]
        default: "-primary"
        maps_to: ".dd-card__link a.dd-button class token"

edit_ui:
  tab_order:
    # parent edit order
    - parent_type
    - sal
    - parent_width

    # child edit order (used when editing an item row)
    - items[].child_image_url
    - items[].child_image_alt
    - items[].child_title
    - items[].child_subtitle
    - items[].child_copy
    - items[].child_link_url
    - items[].child_link_target
    - items[].child_link_label

  navigation_tree:
    parent_row: "dd-card"
    child_rows: "items[]"
    item_row_label: "item {index}: child_title"
    collapse_expand_key: "Space"

  item_collection:
    add_item_key: "A"
    remove_item_key: "X"
    add_behavior: "insert after selected item row, otherwise append to end"
    min_items: 1

  enter_behavior:
    parent_row: "start parent field editing"
    item_row: "start selected items[].child_image_url editing"

  modal_fields:
    parent_edit_modes:
      - parent_type
      - parent_class
      - sal
      - parent_width
    item_edit_modes:
      - items[].child_image_url
      - items[].child_image_alt
      - items[].child_title
      - items[].child_subtitle
      - items[].child_copy
      - items[].child_link_url
      - items[].child_link_target
      - items[].child_link_label
    scope_rule: "when editing an items[] row, parent fields are not editable; when editing parent row, item fields are not editable"
    hide_when_editing_parent_or_child:
      - column.id
      - column.width_class

blueprint:
  label: "dd-card"
  show_fields:
    - "items[active].child_title"
---

## HTML Template

```html
<div class="dd-card [parent_type] [parent_class]">
  <div class="dd-card__items dd-g">
    <!-- repeat: items -->
    <div class="dd-card__item l-box [parent_width]" data-sal="[sal]">
      <div class="dd-card__body dd-g">
        <div class="dd-card__image">
          <img src="[child_image_url]" alt="[child_image_alt]" class="dd-img" loading="lazy">
        </div>
        <div class="dd-card__copy l-box">
          <div class="dd-card__title"><h3>[child_title]</h3></div>
          <div class="dd-card__subtitle"><strong>[child_subtitle]</strong></div>
          [[child_copy_html]]
          <div class="dd-card__links dd-g">
            <div class="dd-card__link">
              <a href="[child_link_url]" target="[child_link_target]" class="dd-button [child_link_style]">[child_link_label]</a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<div class="dd-card [parent_type]">
  <div class="dd-card__items dd-g"><!-- cards loop inside items -->
    <!-- repeat: items -->
    <div class="dd-card__item l-box [parent_width]" data-sal="[sal]">
      <div class="dd-card__body dd-g">
        <div class="dd-card__image">
          <img src="[child_image_url]" alt="[child_image_alt]" class="dd-img" loading="lazy">
        </div>
        <div class="dd-card__copy l-box">
          <div class="dd-card__title">
            <h3>[child_title]</h3>
          </div>
          <div class="dd-card__subtitle">
            <strong>[child_subtitle]</strong>
          </div>
          [[child_copy_html]]
          <div class="dd-card__links dd-g">
            <div class="dd-card__link">
              <a href="[child_image_url]" target="[child_link_target]" class="dd-button -primary">[child_link_label]</a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
```

## Conditional Markup

- render `.dd-card__links` only when both `child_link_url` and `child_link_label` are non-empty
- when `child_link_target` is empty, default to `_self`
