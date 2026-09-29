---
component: dd-data-table
version: 1
node_scope: section_item
insert:
  defaults:
    caption: "Data table"
    dense: false
    columns:
      - label: "ID"
        align: start
        sortable: false
      - label: "Status"
        align: start
        sortable: false
    rows:
      - cells:
          - type: text
            text: "ROW-001"
          - type: badge
            text: "Info"
            badge: "-info"
fields:
  - id: caption
    required: true
    type: string
    maps_to: ".dd-data-table__caption"
  - id: dense
    required: false
    type: bool
    default: false
    maps_to: ".dd-data-table.-dense"
  - id: scroll_label
    required: false
    type: string
    default: "{caption}, scrollable"
    maps_to: ".dd-data-table__scroll[data-label]"
    notes: "JS adds tabindex/role/aria-label only when the table overflows."
  - id: empty_message
    required: false
    type: string
    default: "No data to display."
    maps_to: ".dd-data-table__row.-empty"
  - id: columns
    required: true
    type: array
    min_items: 1
    max_items: 5
    item_fields:
      - id: label
        required: true
        type: string
        maps_to: ".dd-data-table__th"
      - id: align
        required: true
        type: enum
        options: ["start", "center", "end"]
        default: start
        maps_to: "[data-align]"
      - id: sortable
        required: false
        type: bool
        default: false
        maps_to: "button.dd-data-table__sort"
        notes: "sort_key is derived from the label slug; fallback col-{index}."
  - id: rows
    required: false
    type: array
    min_items: 0
    item_fields:
      - id: cells
        required: false
        type: array
        max_items: 5
        item_fields:
          - id: type
            required: true
            type: enum
            options: ["text", "badge"]
            default: text
          - id: text
            required: false
            type: string
            maps_to: ".dd-data-table__td / .dd-badge__label"
          - id: badge
            required: false
            type: enum
            options: ["-critical", "-warning", "-info", "-pass"]
            default: "-info"
            visible_when: "type == badge"
            maps_to: ".dd-badge"
edit_ui:
  tab_order:
    - caption
    - dense
    - scroll_label
    - empty_message
    - columns[].label
    - columns[].align
    - columns[].sortable
    - rows[].cells[].type
    - rows[].cells[].text
    - rows[].cells[].badge
blueprint:
  label: "dd-data-table"
  show_fields:
    - caption
    - columns[].label
    - rows[].cells[0].text
---

## HTML Template

```html
<div class="dd-data-table [.-dense]">
  <div class="dd-data-table__scroll" data-label="[scroll_label]">
    <table class="dd-data-table__table">
      <caption class="dd-data-table__caption">[caption]</caption>
      <thead>
        <tr>
          <!-- sortable: button.dd-data-table__sort[data-sort-key] inside th[aria-sort=none] -->
          <th scope="col" class="dd-data-table__th" data-align="[align]">[label]</th>
        </tr>
      </thead>
      <tbody>
        <tr class="dd-data-table__row">
          <th scope="row" class="dd-data-table__td" data-align="[align]">[first cell]</th>
          <td class="dd-data-table__td"><span class="dd-badge [badge]"><span class="dd-badge__label">[text]</span></span></td>
        </tr>
        <!-- empty: tr.dd-data-table__row.-empty td[colspan] -->
      </tbody>
    </table>
  </div>
</div>
```
