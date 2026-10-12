function dd_navigation() {

  const dd_menu_toggle = document.querySelector('.dd-menu__toggle');
  const dd_menu_close = document.querySelector('.dd-menu__close');
  const dd_main_menu = document.querySelector('.navigation.-main-menu');
  const desktop_query = window.matchMedia('(min-width: 64em)');
  const fine_pointer_query = window.matchMedia('(hover: hover) and (pointer: fine)');
  let submenu_sequence = 0;

  function is_desktop_nav() {
    return desktop_query.matches;
  }

  function is_mobile_open() {
    return !!(dd_main_menu && dd_main_menu.classList.contains('-active') && !is_desktop_nav());
  }

  function direct_child(item, tag_name) {
    const children = item.children;
    for (let i = 0; i < children.length; i += 1) {
      if (children[i].tagName === tag_name) return children[i];
    }
    return null;
  }

  function direct_toggle(item) {
    const children = item.children;
    for (let i = 0; i < children.length; i += 1) {
      if (children[i].classList && children[i].classList.contains('menu-item__toggle')) {
        return children[i];
      }
    }
    return null;
  }

  function direct_submenu(item) {
    const children = item.children;
    for (let i = 0; i < children.length; i += 1) {
      if (children[i].classList && children[i].classList.contains('sub-menu')) {
        return children[i];
      }
    }
    return null;
  }

  function item_name(link) {
    if (!link) return 'section';
    const name = link.textContent.replace(/\s+/g, ' ').trim();
    return name || 'section';
  }

  function next_submenu_id() {
    submenu_sequence += 1;
    let id = 'nav-submenu-' + submenu_sequence;
    while (document.getElementById(id)) {
      submenu_sequence += 1;
      id = 'nav-submenu-' + submenu_sequence;
    }
    return id;
  }

  function append_caret(toggle) {
    if (toggle.querySelector('.menu-item__caret')) return;
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('class', 'menu-item__caret');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('width', '20');
    svg.setAttribute('height', '20');
    svg.setAttribute('aria-hidden', 'true');
    svg.setAttribute('focusable', 'false');
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('fill', 'currentColor');
    path.setAttribute('d', 'M7.41 8.59 12 13.17l4.59-4.58L18 10l-6 6-6-6z');
    svg.appendChild(path);
    toggle.appendChild(svg);
  }

  function ensure_toggle(item, link, submenu) {
    let toggle = direct_toggle(item);
    if (!toggle) {
      toggle = document.createElement('button');
      toggle.type = 'button';
      toggle.className = 'menu-item__toggle';
      const label = document.createElement('span');
      label.className = 'visually-hidden';
      toggle.appendChild(label);
      item.insertBefore(toggle, submenu);
    }
    append_caret(toggle);
    if (!submenu.id) submenu.id = next_submenu_id();
    toggle.setAttribute('aria-controls', submenu.id);
    if (!toggle.hasAttribute('aria-expanded')) {
      toggle.setAttribute('aria-expanded', 'false');
    }
    const hidden = toggle.querySelector('.visually-hidden');
    if (hidden && !hidden.textContent.trim()) {
      hidden.textContent = 'Show ' + item_name(link) + ' submenu';
    }
    return toggle;
  }

  function focus_control(el) {
    if (!el || typeof el.focus !== 'function') return;
    el.focus();
    if (typeof el.scrollIntoView === 'function') {
      el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }
  }

  // aria-expanded lives only on the disclosure button. Every open and close path uses this.
  function set_expanded(item, expanded) {
    const toggle = direct_toggle(item);
    const link = direct_child(item, 'A');
    item.classList.toggle('-active', expanded);
    if (toggle) {
      toggle.setAttribute('aria-expanded', expanded ? 'true' : 'false');
      const hidden = toggle.querySelector('.visually-hidden');
      if (hidden) {
        hidden.textContent = (expanded ? 'Hide ' : 'Show ') + item_name(link) + ' submenu';
      }
    }
    if (!expanded) {
      const nested = item.querySelectorAll('.menu-item.-active');
      for (let i = 0; i < nested.length; i += 1) {
        if (nested[i] !== item) set_expanded(nested[i], false);
      }
    }
  }

  function open_item(item) {
    const list = item.parentElement;
    if (list) {
      const siblings = list.children;
      for (let i = 0; i < siblings.length; i += 1) {
        const sibling = siblings[i];
        if (sibling !== item && sibling.classList && sibling.classList.contains('-active')) {
          set_expanded(sibling, false);
        }
      }
    }
    set_expanded(item, true);
  }

  function close_submenus(root) {
    const scope = root || document;
    const open_items = scope.querySelectorAll('.navigation.-main-menu .menu-item.-active');
    for (let i = open_items.length - 1; i >= 0; i -= 1) {
      const item = open_items[i];
      if (item.contains(document.activeElement)) {
        const toggle = direct_toggle(item);
        if (toggle) focus_control(toggle);
      }
      set_expanded(item, false);
    }
  }

  function list_items(list) {
    const items = [];
    if (!list) return items;
    const children = list.children;
    for (let i = 0; i < children.length; i += 1) {
      if (children[i].classList && children[i].classList.contains('menu-item')) {
        items.push(children[i]);
      }
    }
    return items;
  }

  function focus_item(item) {
    if (!item) return;
    const link = direct_child(item, 'A');
    const toggle = direct_toggle(item);
    focus_control(link || toggle);
  }

  function is_visible(el) {
    return !!(el.offsetWidth || el.offsetHeight || el.getClientRects().length);
  }

  function visible_controls(nav) {
    const controls = nav.querySelectorAll('.menu a[href], .menu .menu-item__toggle');
    const visible = [];
    for (let i = 0; i < controls.length; i += 1) {
      if (is_visible(controls[i])) visible.push(controls[i]);
    }
    return visible;
  }

  function move_visible(current, delta) {
    const nav = current.closest('.navigation.-main-menu');
    if (!nav) return;
    const controls = visible_controls(nav);
    const index = controls.indexOf(current);
    if (index === -1) return;
    const next = controls[index + delta];
    if (next) focus_control(next);
  }

  function open_and_focus(item, which) {
    const submenu = direct_submenu(item);
    if (!submenu) return;
    open_item(item);
    const items = list_items(submenu);
    focus_item(which === 'last' ? items[items.length - 1] : items[0]);
  }

  function closest_open_item(node) {
    if (!node || !node.closest) return null;
    let current = node.closest('.menu-item');
    while (current) {
      if (current.classList.contains('-active')) return current;
      const parent = current.parentElement;
      current = parent ? parent.closest('.menu-item') : null;
    }
    return null;
  }

  function on_menu_keydown(event) {
    const key = event.key;
    if (key !== 'ArrowDown' && key !== 'ArrowUp' && key !== 'ArrowLeft' && key !== 'ArrowRight' && key !== 'Home' && key !== 'End') {
      return;
    }
    const target = event.target;
    if (!target.closest) return;
    const item = target.closest('.menu-item');
    if (!item || !target.closest('.navigation.-main-menu')) return;
    const is_toggle = target.classList && target.classList.contains('menu-item__toggle');
    if (target.tagName !== 'A' && !is_toggle) return;

    const submenu = direct_submenu(item);
    const collapsed = !!(submenu && !item.classList.contains('-active'));
    const on_parent_control = target === direct_child(item, 'A') || target === direct_toggle(item);

    if ((key === 'ArrowDown' || key === 'ArrowRight') && collapsed && on_parent_control) {
      event.preventDefault();
      open_and_focus(item, 'first');
      return;
    }

    if (key === 'ArrowDown' || key === 'ArrowRight') {
      event.preventDefault();
      move_visible(target, 1);
      return;
    }

    if (key === 'ArrowUp' || key === 'ArrowLeft') {
      event.preventDefault();
      move_visible(target, -1);
      return;
    }

    if (key === 'Home' || key === 'End') {
      event.preventDefault();
      const items = list_items(item.parentElement);
      if (!items.length) return;
      focus_item(key === 'Home' ? items[0] : items[items.length - 1]);
    }
  }

  function trap_root() {
    if (!dd_main_menu) return null;
    return dd_main_menu.closest('.dd-header') || dd_main_menu;
  }

  function focusable_in(root) {
    const nodes = root.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])');
    const visible = [];
    for (let i = 0; i < nodes.length; i += 1) {
      if (is_visible(nodes[i])) visible.push(nodes[i]);
    }
    return visible;
  }

  function on_document_keydown(event) {
    if (event.target.closest && event.target.closest('dialog')) return;
    if (document.querySelector('dialog[open]')) return;

    if (event.key === 'Tab' && is_mobile_open()) {
      const root = trap_root();
      if (!root) return;
      const items = focusable_in(root);
      if (!items.length) return;
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (event.shiftKey && (active === first || !root.contains(active))) {
        event.preventDefault();
        focus_control(last);
      } else if (!event.shiftKey && (active === last || !root.contains(active))) {
        event.preventDefault();
        focus_control(first);
      }
      return;
    }

    if (event.key !== 'Escape') return;

    const open_item = closest_open_item(event.target);
    if (open_item) {
      event.preventDefault();
      const toggle = direct_toggle(open_item);
      if (toggle) focus_control(toggle);
      set_expanded(open_item, false);
      return;
    }

    if (is_mobile_open()) {
      event.preventDefault();
      dd_navigation_set_open(false);
    }
  }

  function setup_parent(item) {
    if (item.dataset.submenuReady === 'true') return;
    const submenu = direct_submenu(item);
    if (!submenu) return;
    item.dataset.submenuReady = 'true';
    item.classList.add('-has-children');

    const link = direct_child(item, 'A');
    const toggle = ensure_toggle(item, link, submenu);
    set_expanded(item, false);

    toggle.addEventListener('click', (event) => {
      event.preventDefault();
      if (item.classList.contains('-active')) set_expanded(item, false);
      else open_item(item);
    });

    item.addEventListener('focusout', (event) => {
      const next = event.relatedTarget;
      if (!next || item.contains(next)) return;
      set_expanded(item, false);
    });

    if (fine_pointer_query.matches) {
      // Hover opens from the link or panel. Hovering the toggle does not, so a
      // click can still open a closed submenu instead of undoing a hover-open.
      item.addEventListener('mouseover', (event) => {
        if (toggle.contains(event.target)) return;
        if (!item.classList.contains('-active')) open_item(item);
      });
      item.addEventListener('mouseleave', () => {
        if (!item.contains(document.activeElement)) set_expanded(item, false);
      });
    }
  }

  function setup_menu(root) {
    const items = root.querySelectorAll('.menu-item');
    for (let i = 0; i < items.length; i += 1) {
      if (direct_submenu(items[i])) setup_parent(items[i]);
    }
    // Buttons exist before this class, so :focus-within can still open panels if JS stops early.
    root.classList.add('-submenu-ready');
    root.addEventListener('keydown', on_menu_keydown);
  }

  function dd_navigation_set_open(is_open) {
    if (!is_open) {
      const should_return = !is_desktop_nav() && dd_menu_toggle && dd_main_menu && dd_main_menu.contains(document.activeElement);
      if (should_return) focus_control(dd_menu_toggle);
      close_submenus(dd_main_menu);
    }

    if (dd_menu_toggle) {
      dd_menu_toggle.classList.toggle('-active', is_open);
      dd_menu_toggle.setAttribute('aria-expanded', is_open ? 'true' : 'false');
    }
    if (dd_main_menu) {
      dd_main_menu.classList.toggle('-active', is_open);
    }
    // dd_search_close() clears body.-active, so set the scroll lock after that call.
    if (!document.querySelector('dialog[open]')) {
      document.body.classList.toggle('-active', is_open);
    }

    if (is_open && !is_desktop_nav() && dd_main_menu) {
      const first = dd_main_menu.querySelector('.menu > .menu-item > a[href]');
      if (first) focus_control(first);
    }
  }

  if (dd_main_menu) {
    if (!dd_main_menu.id) dd_main_menu.id = 'header-navigation';
    if (dd_menu_toggle && !dd_menu_toggle.getAttribute('aria-controls')) {
      dd_menu_toggle.setAttribute('aria-controls', dd_main_menu.id);
    }
  }

  const close_labels = document.querySelectorAll('.navigation.-main-menu .dd-menu__close .visually-hidden');
  for (let i = 0; i < close_labels.length; i += 1) {
    close_labels[i].textContent = 'Close menu';
  }

  if (dd_menu_toggle) {
    dd_menu_toggle.setAttribute('aria-expanded', 'false');
    dd_menu_toggle.addEventListener('click', () => {
      const will_open = !dd_menu_toggle.classList.contains('-active');
      if (will_open && typeof dd_search_close === 'function') {
        dd_search_close();
      }
      dd_navigation_set_open(will_open);
    });
  }

  if (dd_menu_close) {
    dd_menu_close.addEventListener('click', () => {
      dd_navigation_set_open(false);
    });
  }

  const menus = document.querySelectorAll('.navigation.-main-menu');
  for (let i = 0; i < menus.length; i += 1) {
    setup_menu(menus[i]);
  }

  document.addEventListener('keydown', on_document_keydown);
  document.addEventListener('click', (event) => {
    const open_items = document.querySelectorAll('.navigation.-main-menu .menu-item.-active');
    for (let i = open_items.length - 1; i >= 0; i -= 1) {
      const item = open_items[i];
      if (item.contains(event.target)) continue;
      if (item.contains(document.activeElement)) {
        const toggle = direct_toggle(item);
        if (toggle) focus_control(toggle);
      }
      set_expanded(item, false);
    }
  });

  if (typeof desktop_query.addEventListener === 'function') {
    desktop_query.addEventListener('change', (event) => {
      if (!event.matches || document.querySelector('dialog[open]')) return;
      if (dd_main_menu && dd_main_menu.classList.contains('-active')) {
        dd_navigation_set_open(false);
      }
    });
  }
}

document.addEventListener('DOMContentLoaded', () => {
  dd_navigation();
});
