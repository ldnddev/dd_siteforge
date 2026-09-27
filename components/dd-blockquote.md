---
component: dd-blockquote
version: 1
node_scope: section_item   # one of: page_node | section_item

insert:
  defaults:
    sal: "fade"
    parent_image_url: "https://dummyimage.com/512x512/000/fff"
    parent_image_alt: "blockquote Persons Name"
    parent_name: "blockquote Persons Name"
    parent_title: "blockquote Persons Title"
    parent_copy: "Copy"

fields:
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "fade"
    maps_to: ".dd-blockquote[data-sal]"

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

  - id: parent_image_url
    required: true
    type: string
    maps_to: ".dd-blockquote__image img[src]"

  - id: parent_image_alt
    required: true
    type: string
    maps_to: ".dd-blockquote__image img[alt]"

  - id: parent_name
    required: true
    type: string
    maps_to: ".dd-blockquote__name"

  - id: parent_title
    required: true
    type: string
    maps_to: ".dd-blockquote__title"

  - id: parent_copy
    required: true
    type: string
    accepts: ["markdown", "html"]
    export_transform: "render markdown to html (raw html passthrough)"
    maps_to: ".dd-blockquote__copy"
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
    - sal
    - parent_image_url
    - parent_image_alt
    - parent_name
    - parent_title
    - parent_copy

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - sal
      - parent_image_url
      - parent_image_alt
      - parent_name
      - parent_title
      - parent_copy
    hide_when_editing_component:
      - column.id
      - column.width_class

blueprint:
  label: "dd-blockquote"
  show_fields:
    - "parent_name"
    - "parent_title"
---

## HTML Template

```html
<blockquote class="dd-blockquote">
  <div class="dd-blockquote__content dd-g" data-sal="[sal]">
    <div class="dd-blockquote__icon"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-quote-icon lucide-quote"><path d="M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"/><path d="M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"/></svg></div>
    <div class="dd-blockquote__person dd-g l-box">
      <div class="dd-blockquote__image">
        <img src="[parent_image_url]" class="dd-img" alt="[parent_image_alt]" loading="lazy" />
      </div>
      <div class="dd-blockquote__name-title">
        <span class="dd-blockquote__name">[parent_name]</span>
        <span class="dd-blockquote__title">, [parent_title]</span>
      </div>
      <div class="dd-blockquote__comment">
        [[parent_copy_html]]
      </div>
    </div>
  </div>
</blockquote>
<script type="application/ld+json">
{
  "@context": "https://schema.org/",
  "@type": "Quotation",
  "creator": {
    "@type": "Person",
    "name": "[parent_name], [parent_title]"
  },
  "text": "[parent_copy]"
}
</script>
```

## Conditional Markup

- none (this variant intentionally has no optional link fields)
