---
component: dd-search-results
version: 1
node_scope: section_item

insert:
  defaults:
    sal: "no-animation"

fields:
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-search-page[data-sal]"
    notes: "Insert and missing JSON sal are no-animation so the form does not animate unless the author picks a style."

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


edit_ui:
  tab_order:
    - sal

  enter_behavior:
    parent_row: "start component field editing"

  modal_fields:
    parent_edit_modes:
      - sal

blueprint:
  label: "dd-search-results"
  show_fields: []
---

## HTML Template

```html
<div class="dd-search-page" data-sal="[sal]">
  <h1 class="dd-search-page__title">Search</h1>
  <form class="dd-search-page__form" action="/search/" method="get" role="search">
    <label class="dd-search-page__label" for="dd-page-search-q">Search the site</label>
    <div class="dd-search-page__field">
      <input class="dd-search-page__input" type="search" id="dd-page-search-q" name="q" autocomplete="off" spellcheck="false" placeholder="Search pages, services, and insights" aria-describedby="dd-page-search-status">
      <button class="dd-search-page__submit dd-button -primary" type="submit">Search</button>
    </div>
  </form>
  <p class="dd-search-page__status" id="dd-page-search-status" role="status"></p>
  <ul class="dd-search-page__results" hidden aria-label="Search results"></ul>
</div>
```

## Conditional Markup

- always renders when present in a page section column
- heading, form action, and placeholder are kit-owned in `templates/dd-search-results.hbs`
- `source/js/components/_dd_search.js` binds `#dd-page-search-q`, status, and the results list against `search-index.json`
- layout width comes from the parent column's `width_class`

## Validation Rules

- `sal` required; must be one of the enum options
- valid in page, header, and footer section columns
