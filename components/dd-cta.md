---
component: dd-cta
version: 1
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    parent_class: "-top-left"
    parent_image_url: "https://dummyimage.com/1920x1080/000000/fff"
    parent_image_alt: "Image alt"
    sal: "no-animation"
    parent_title: "Title"
    parent_subtitle: "Subtitle"
    parent_copy: "Copy"
    links:
      - { url: "/path", label: "Learn More", target: "_self", style: "-primary" }

fields:
  - id: parent_class
    required: true
    type: enum
    options: ["-top-left", "-top-center", "-top-right", "-center-left", "-center-center", "-center-right", "-bottom-left", "-bottom-center", "-bottom-right"]
    default: "-top-left"
    maps_to: ".dd-cta class token"
    
  - id: parent_image_url
    required: true
    type: string
    default: "https://dummyimage.com/1920x1080/000000/fff"
    maps_to: ".dd-cta__image img[src], .dd-cta__image[style background-image]"
    
  - id: parent_image_alt
    required: true
    type: string
    default: "Image alt"
    maps_to: ".dd-cta__image img[alt]"

  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-cta__content[data-sal]"

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
    maps_to: ".dd-cta__title"

  - id: parent_subtitle
    required: true
    type: string
    maps_to: ".dd-cta__subtitle"

  - id: parent_copy
    required: true
    type: string
    accepts: ["markdown", "html"]
    export_transform: "render markdown to html (raw html passthrough)"
    maps_to: ".dd-cta__copy"
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

  - id: links
    required: false
    type: array
    min_items: 0
    max_items: 4
    item_fields:
      - id: url
        required: true
        type: string
        maps_to: ".dd-cta__link a[href]"
      - id: label
        required: true
        type: string
        maps_to: ".dd-cta__link a"
      - id: target
        required: false
        type: enum
        options: ["_self", "_blank"]
        default: "_self"
        maps_to: ".dd-cta__link a[target]"
      - id: style
        required: true
        type: enum
        options: ["-primary", "-secondary", "-tertiary", "-ghost"]
        default: "-primary"
        maps_to: ".dd-cta__link a.dd-button class token"

edit_ui:
  tab_order:
    - parent_class
    - parent_image_url
    - parent_image_alt
    - sal
    - parent_title
    - parent_subtitle
    - parent_copy
    - links

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - parent_class
      - parent_image_url
      - parent_image_alt
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
  label: "dd-cta"
  show_fields:
    - "parent_title"
---

## HTML Template

```html
<div class="dd-cta [parent_class]">
  <div class="dd-cta__image" style="background-image: url([parent_image_url]);">
    <picture>
      <img src="[parent_image_url]" class="dd-img" alt="[parent_image_alt]" />
    </picture>
  </div>
  <div class="dd-cta__content dd-g" data-sal="[sal]">
    <div class="dd-cta__copy dd-u-1-1 dd-u-md-12-24">
      <div class="dd-cta__title">
        <h2>[parent_title]</h2>
      </div>
      <div class="dd-cta__subtitle">
        <strong>[parent_subtitle]</strong>
      </div>
      [[parent_copy_html]]
      <div class="dd-cta__links dd-g -x-center">
        <div class="dd-cta__link">
          <a href="[url]" class="dd-button [style]" target="[target]">[label]</a>
        </div>
      </div>
    </div>
  </div>
</div>
```

## Conditional Markup

- render `.dd-cta__links` only when `links` is non-empty
- each link requires url and label; target defaults to `_self`; style defaults to `-primary`
