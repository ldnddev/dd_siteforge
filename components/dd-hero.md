---
component: dd-hero
version: 1
node_scope: page_node
insert:
  defaults:
    hero_class: "-full-full"
    sal: "no-animation"
    custom_css: ""
    image: "https://dummyimage.com/1920x1080/000/fff"
    title: "Build with dd-framework"
    subtitle: "Framework-native static page builder"
    copy: "Compose pages with typed component schemas."
    links:
      - { url: "/start", label: "Get Started", target: "_self", style: "-primary" }
      - { url: "/learn-more", label: "Learn More", target: "_self", style: "-ghost" }
    overlay: ""
    copy_position: "-left"
    id: ""
    aria_label: "Introduction"
fields:
  - id: image
    required: true
    type: string
    maps_to: ".dd-hero__image img[src]"
  - id: hero_class
    required: true
    type: enum
    options: ["-contained", "-contained-md", "-contained-lg", "-contained-xl", "-contained-xxl", "-full-full", "-full-contained", "-full-contained-md", "-full-contained-lg", "-full-contained-xl", "-full-contained-xxl"]
    default: "-full-full"
    maps_to: ".dd-hero class token"
  - id: sal
    required: true
    type: enum
    options: ["no-animation","fade","slide-up","slide-down","slide-left","slide-right","zoom-in","zoom-out","flip-up","flip-down","flip-left","flip-right"]
    default: "no-animation"
    maps_to: ".dd-hero__content[data-sal]"

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

  - id: custom_css
    required: false
    type: string
    default: ""
    maps_to: ".dd-hero class token"
  - id: title
    required: true
    type: string
    maps_to: ".dd-hero__title h1"
  - id: subtitle
    required: false
    type: string
    maps_to: ".dd-hero__subtitle"
  - id: copy
    required: false
    type: textarea
    rows: 3
    accepts: ["markdown", "html"]
    export_transform: "render markdown to html (raw html passthrough)"
    maps_to: ".dd-hero__body"
  - id: links
    required: false
    type: array
    min_items: 0
    max_items: 2
    item_fields:
      - id: url
        required: true
        type: string
        maps_to: ".dd-hero__link a[href]"
      - id: label
        required: true
        type: string
        maps_to: ".dd-hero__link a"
      - id: target
        required: false
        type: enum
        options: ["_self", "_blank"]
        default: "_self"
        maps_to: ".dd-hero__link a[target]"
      - id: style
        required: true
        type: enum
        options: ["-primary", "-secondary", "-tertiary", "-ghost"]
        default: "-primary"
        maps_to: ".dd-hero__link a.dd-button class token"

  - id: overlay
    required: false
    type: enum
    options: ["", "-overlay-light", "-overlay-dark"]
    default: ""
    maps_to: ".dd-hero class token"

  - id: copy_position
    required: true
    type: enum
    options: ["-left", "-center", "-right"]
    default: "-left"
    maps_to: ".dd-hero class token"
    notes: "Missing JSON becomes -left on next save. Separate from width parent_class."

  - id: id
    required: false
    type: string
    default: ""
    maps_to: "section[id] when set"
    notes: "HTML-safe slug; unique per page among heroes and sections. Omitted from HTML when empty."

  - id: aria_label
    required: false
    type: string
    default: "Introduction"
    maps_to: "section[aria-label]"
    notes: "HTML falls back to Introduction when empty."
edit_ui:
  tab_order:
    - image
    - hero_class
    - sal
    - custom_css
    - title
    - subtitle
    - copy
    - links
    - overlay
    - copy_position
    - id
    - aria_label
blueprint:
  label: "dd-hero"
  show_fields:
    - hero_class
    - overlay
    - copy_position
    - sal
    - custom_css
    - title
    - subtitle
    - links
    - image
---

## HTML Template

```html
<section class="dd-hero [hero_class] [overlay] [copy_position] [custom_css]" id="[id]" aria-label="[aria_label]">
  <div class="dd-hero__image">
    <picture>
      <img src="[image]" class="dd-img" alt="[image_alt]" />
    </picture>
  </div>
  <div class="dd-hero__content dd-g" data-sal="[sal]">
    <div class="dd-hero__copy dd-u-1-1 dd-u-lg-12-24">
      <div class="dd-hero__title"><h1>[title]</h1></div>
      <!-- if [subtitle] --><div class="dd-hero__subtitle"><strong>[subtitle]</strong></div><!-- endif -->
      <!-- if [copy] --><div class="dd-hero__body"><p>[copy]</p></div><!-- endif -->
      <!-- if links --><div class="dd-hero__links dd-g">
        <!-- each link --><div class="dd-hero__link"><a href="[url]" class="dd-button [style]" target="[target]">[label]</a></div><!-- endeach -->
      </div><!-- endif -->
    </div>
  </div>
</section>
```
