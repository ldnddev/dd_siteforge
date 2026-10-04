---
component: dd-timeline
version: 1
node_scope: section_item
insert:
  defaults:
    aria_label: "Timeline"
    sal: "no-animation"
    items:
      - child_year: "2024"
        child_title: "Title"
        heading_level: 3
        child_copy: "Copy"
fields:
  - id: aria_label
    required: false
    type: string
    default: "Timeline"
    maps_to: ".dd-timeline[aria-label]"
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-timeline__item[data-sal]"
    notes: "Stagger delay 100×index, cap 1000."
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
      - id: child_year
        required: true
        type: string
        maps_to: ".dd-timeline__year time"
      - id: child_datetime
        required: false
        type: string
        maps_to: "time[datetime]"
        notes: "Required when year is not YYYY, YYYY-MM, or YYYY-MM-DD."
      - id: child_title
        required: true
        type: string
        maps_to: ".dd-timeline__title"
      - id: heading_level
        required: true
        type: enum
        options: [2, 3, 4, 5, 6]
        default: 3
        maps_to: ".dd-timeline__title heading tag"
      - id: child_copy
        required: true
        type: textarea
        accepts: ["markdown", "html"]
        export_transform: "render markdown to html (raw html passthrough)"
        maps_to: ".dd-timeline__copy"
      - id: child_image_url
        required: false
        type: string
        maps_to: ".dd-timeline__image img[src]"
      - id: child_image_alt
        required: false
        type: string
        maps_to: ".dd-timeline__image img[alt]"
edit_ui:
  tab_order:
    - aria_label
    - sal
    - items[].child_year
    - items[].child_datetime
    - items[].child_title
    - items[].heading_level
    - items[].child_copy
    - items[].child_image_url
    - items[].child_image_alt
blueprint:
  label: "dd-timeline"
  show_fields:
    - items[].child_year
    - items[].child_title
---

## HTML Template

```html
<div class="dd-timeline" role="region" aria-label="[aria_label]">
  <div class="dd-timeline__content">
    <ol class="dd-timeline__items dd-g">
      <!-- each event -->
      <li class="dd-timeline__item dd-u-1-1" data-sal="[sal]">
        <div class="dd-timeline__body dd-g">
          <!-- if image --><div class="dd-timeline__image l-box dd-u-1-1"><img src="[url]" alt="[alt]" class="dd-img" /></div><!-- endif -->
          <div class="dd-timeline__text l-box dd-u-1-1">
            <div class="dd-timeline__year"><time datetime="[datetime]">[year]</time></div>
            <div class="dd-timeline__title"><h[level]>[title]</h[level]></div>
            <div class="dd-timeline__copy">[[child_copy_html]]</div>
          </div>
        </div>
      </li>
      <!-- endeach -->
    </ol>
  </div>
</div>
```
