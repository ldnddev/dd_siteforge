---
component: dd-spacer
version: 1
node_scope: section_item
insert:
  defaults:
    size: "-md"
    divider: false
fields:
  - id: size
    required: true
    type: enum
    options: ["-sm", "-md", "-lg", "-xl", "-xxl", "-xxxl"]
    default: "-md"
    maps_to: ".dd-spacer class token"
  - id: divider
    required: false
    type: enum
    options: ["false", "true"]
    default: "false"
    maps_to: ".dd-spacer.-divider"
edit_ui:
  tab_order:
    - size
    - divider
blueprint:
  label: "dd-spacer"
  show_fields:
    - size
    - divider
---

## HTML Template

```html
<div class="dd-spacer [size] [divider]" aria-hidden="true"></div>
```

Presentational whitespace. `-divider` is a decorative line, not a thematic `<hr>`.
