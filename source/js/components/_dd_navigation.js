function dd_navigation() {

  const dd_menu_toggle = document.querySelector('.dd-menu__toggle');
  const dd_menu_close = document.querySelector('.dd-menu__close');
  const dd_main_menu = document.querySelector('.navigation.-main-menu');
  const desktop_query = window.matchMedia('(min-width: 64em)');
  const fine_pointer_query = window.matchMedia('(hover: hover) and (pointer: fine)');
  let submenu_sequence = 0;
  let ignore_dead_click = false;

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

  function direct_by_class(item, class_name) {
    const children = item.children;
    for (let i = 0; i < children.length; i += 1) {
      if (children[i].classList && children[i].classList.contains(class_name)) {
        return children[i];
      }
    }
    return null;
  }

  function direct_toggle(item) {
    return direct_by_class(item, 'menu-item__toggle');
  }

  function direct_submenu(item) {
    return direct_by_class(item, 'sub-menu');
  }

  function item_name(link) {
    if (!link) return 'section';
    const name = link.textContent.replace(/\s+/g, ' ').trim();
    return name || 'section';
  }

  function previous_menu_name(item) {
    const list = item.parentElement;
    if (list && list.classList.contains('sub-menu')) {
      const parent_item = list.parentElement;
      const parent_link = parent_item ? direct_child(parent_item, 'A') : null;
      return item_name(parent_link);
    }
    return 'menu';
  }

  function is_dead_link(link) {
    if (!link) return false;
    if (link.classList.contains('-no-link')) return true;
    const href = link.getAttribute('href');
    return href === '' || href === '#';
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

  function make_svg(class_name, path_d) {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('class', class_name);
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('width', '24');
    svg.setAttribute('height', '24');
    svg.setAttribute('aria-hidden', 'true');
    svg.setAttribute('focusable', 'false');
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('fill', 'currentColor');
    path.setAttribute('d', path_d);
    svg.appendChild(path);
    return svg;
  }

  function append_caret(toggle) {
    if (toggle.querySelector('.menu-item__caret')) return;
    toggle.appendChild(make_svg('menu-item__caret', 'M7.41 8.59 12 13.17l4.59-4.58L18 10l-6 6-6-6z'));
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
    toggle.setAttribute('aria-expanded', 'false');
    const hidden = toggle.querySelector('.visually-hidden');
    if (hidden) hidden.textContent = 'Show ' + item_name(link) + ' submenu';
    return toggle;
  }

  function ensure_panel_chrome(item, submenu, link) {
    let back_item = direct_by_class(submenu, 'menu-item--chrome-back');
    if (!back_item) {
      back_item = document.createElement('li');
      back_item.className = 'menu-item menu-item--chrome-back';
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'menu-item__back';
      const text = document.createElement('span');
      text.className = 'menu-item__back-text';
      text.textContent = 'Back';
      button.appendChild(make_svg('menu-item__back-icon', 'M15.41 16.59 10.83 12l4.58-4.59L14 6l-6 6 6 6z'));
      button.appendChild(text);
      back_item.appendChild(button);
      submenu.insertBefore(back_item, submenu.firstChild);
      button.addEventListener('click', (event) => {
        event.preventDefault();
        close_panel(item);
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

    let label_item = direct_by_class(submenu, 'menu-item--chrome-label');
    if (!label_item) {
      label_item = document.createElement('li');
      label_item.className = 'menu-item menu-item--chrome-label';
      const heading = document.createElement('span');
      heading.className = 'menu-item__label';
      heading.setAttribute('role', 'heading');
      heading.setAttribute('aria-level', '2');
      label_item.appendChild(heading);
      submenu.insertBefore(label_item, back_item.nextSibling);
    }

    const back_button = back_item.querySelector('.menu-item__back');
    if (back_button) back_button.setAttribute('aria-label', 'Back to ' + previous_menu_name(item));
    const heading = label_item.querySelector('.menu-item__label');
    if (heading) heading.textContent = item_name(link);
    submenu.setAttribute('aria-label', item_name(link));
  }

  function focus_control(el) {
    if (!el || typeof el.focus !== 'function') return;
    el.focus();
    if (typeof el.scrollIntoView === 'function') {
      el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }
  }

  function sync_panel_box() {
    if (!dd_main_menu || is_desktop_nav() || !dd_main_menu.classList.contains('-active')) return;
    const close = dd_main_menu.querySelector('.dd-menu__close');
    if (!close) return;
    const top = Math.ceil(close.getBoundingClientRect().bottom);
    dd_main_menu.style.setProperty('--dd-nav-panel-top', top + 'px');
  }

  // inert and aria-hidden are applied here only, after open/close classes settle.
  function sync_inert() {
    const desktop = is_desktop_nav();
    const navs = document.querySelectorAll('.navigation.-main-menu');
    for (let n = 0; n < navs.length; n += 1) {
      const nav = navs[n];
      const submenus = nav.querySelectorAll('.sub-menu');
      for (let i = 0; i < submenus.length; i += 1) {
        const submenu = submenus[i];
        const owner = submenu.parentElement;
        const open = !!(owner && owner.classList.contains('menu-item') && owner.classList.contains('-active'));
        if (open) {
          submenu.inert = false;
          submenu.removeAttribute('aria-hidden');
        } else {
          submenu.inert = true;
          submenu.setAttribute('aria-hidden', 'true');
        }
      }

      const covered = nav.querySelectorAll('.menu-item[inert], .menu-item > a[inert], .menu-item__toggle[inert]');
      for (let i = 0; i < covered.length; i += 1) covered[i].removeAttribute('inert');
      if (desktop) continue;

      const lists = nav.querySelectorAll('.menu, .sub-menu');
      for (let i = 0; i < lists.length; i += 1) {
        const list = lists[i];
        if (list.inert || list.hasAttribute('aria-hidden')) continue;
        const kids = child_items(list);
        let open_child = null;
        for (let k = 0; k < kids.length; k += 1) {
          if (kids[k].classList.contains('-active')) {
            open_child = kids[k];
            break;
          }
        }
        if (!open_child) continue;
        for (let k = 0; k < kids.length; k += 1) {
          if (kids[k] !== open_child) kids[k].setAttribute('inert', '');
        }
        const open_link = direct_child(open_child, 'A');
        const open_toggle = direct_toggle(open_child);
        if (open_link) open_link.setAttribute('inert', '');
        if (open_toggle) open_toggle.setAttribute('inert', '');
      }
    }
  }

  function apply_expanded(item, expanded) {
    const toggle = direct_toggle(item);
    const link = direct_child(item, 'A');
    item.classList.toggle('-active', expanded);
    const expanded_value = expanded ? 'true' : 'false';
    const name = item_name(link);
    if (toggle) {
      toggle.setAttribute('aria-expanded', expanded_value);
      const hidden = toggle.querySelector('.visually-hidden');
      if (hidden) hidden.textContent = (expanded ? 'Hide ' : 'Show ') + name + ' submenu';
    }
    if (link && link.getAttribute('role') === 'button') {
      link.setAttribute('aria-expanded', expanded_value);
    }
    if (!expanded) {
      const nested = item.querySelectorAll('.menu-item.-active');
      for (let i = 0; i < nested.length; i += 1) {
        if (nested[i] !== item) apply_expanded(nested[i], false);
      }
    }
  }

  function open_item(item, focus_inside) {
    const list = item.parentElement;
    if (list) {
      const siblings = list.children;
      for (let i = 0; i < siblings.length; i += 1) {
        const sibling = siblings[i];
        if (sibling !== item && sibling.classList && sibling.classList.contains('-active')) {
          apply_expanded(sibling, false);
        }
      }
    }
    apply_expanded(item, true);
    if (!is_desktop_nav()) sync_panel_box();
    sync_inert();
    if (focus_inside) focus_panel_start(item);
  }

  function close_panel(item) {
    const toggle = direct_toggle(item);
    apply_expanded(item, false);
    sync_inert();
    if (toggle) focus_control(toggle);
  }

  function close_submenus(root) {
    const scope = root || document;
    const open_items = scope.querySelectorAll('.navigation.-main-menu .menu-item.-active');
    for (let i = open_items.length - 1; i >= 0; i -= 1) {
      apply_expanded(open_items[i], false);
    }
    sync_inert();
  }

  function child_items(list) {
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

  function is_operable(el) {
    if (!el) return false;
    if (el.inert) return false;
    if (el.closest && el.closest('[inert], [hidden], [aria-hidden="true"]')) return false;
    let node = el;
    while (node && node !== document.documentElement) {
      const style = window.getComputedStyle(node);
      if (style.display === 'none' || style.visibility === 'hidden') return false;
      node = node.parentElement;
    }
    return true;
  }

  function focus_item(item) {
    if (!item) return;
    if (item.classList.contains('menu-item--chrome-back')) {
      focus_control(item.querySelector('.menu-item__back'));
      return;
    }
    focus_control(direct_child(item, 'A') || direct_toggle(item));
  }

  function panel_stops(list) {
    const stops = [];
    if (!list) return stops;
    const children = list.children;
    for (let i = 0; i < children.length; i += 1) {
      const item = children[i];
      if (!item.classList || !item.classList.contains('menu-item')) continue;
      if (item.classList.contains('menu-item--chrome-label')) continue;
      if (item.classList.contains('menu-item--chrome-back')) {
        if (!is_desktop_nav()) {
          const back = item.querySelector('.menu-item__back');
          if (back && is_operable(back)) stops.push(item);
        }
        continue;
      }
      const control = direct_child(item, 'A') || direct_toggle(item);
      if (control && is_operable(control)) stops.push(item);
    }
    return stops;
  }

  function move_in_list(item, delta) {
    const items = panel_stops(item.parentElement);
    const index = items.indexOf(item);
    if (index === -1) return false;
    const next = items[index + delta];
    if (!next) return false;
    focus_item(next);
    return true;
  }

  function focus_panel_start(item) {
    const submenu = direct_submenu(item);
    if (!submenu) return;
    if (!is_desktop_nav()) {
      const back = direct_by_class(submenu, 'menu-item--chrome-back');
      const button = back ? back.querySelector('.menu-item__back') : null;
      if (button) {
        focus_control(button);
        return;
      }
    }
    const stops = panel_stops(submenu);
    if (stops[0]) focus_item(stops[0]);
  }

  function parent_item_of(list) {
    if (!list || !list.classList.contains('sub-menu')) return null;
    const parent_item = list.parentElement;
    if (!parent_item || !parent_item.classList.contains('menu-item')) return null;
    return parent_item;
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

  function toggle_parent(item, focus_inside) {
    if (item.classList.contains('-active')) {
      close_panel(item);
      return;
    }
    open_item(item, focus_inside);
  }

  function on_menu_keydown(event) {
    const key = event.key;
    if (key !== 'ArrowDown' && key !== 'ArrowUp' && key !== 'ArrowLeft' && key !== 'ArrowRight' && key !== 'Home' && key !== 'End') {
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    const target = event.target;
    if (!target.closest || !target.closest('.navigation.-main-menu')) return;
    const item = target.closest('.menu-item');
    if (!item) return;
    if (target.tagName !== 'A' && target.tagName !== 'BUTTON') return;

    const submenu = direct_submenu(item);
    const is_parent_control = target === direct_child(item, 'A') || target === direct_toggle(item);
    const desktop = is_desktop_nav();
    const list = item.parentElement;
    const on_top = !!(list && list.classList.contains('menu') && !list.classList.contains('sub-menu'));

    if (key === 'Home' || key === 'End') {
      const items = panel_stops(list);
      if (!items.length) return;
      event.preventDefault();
      focus_item(key === 'Home' ? items[0] : items[items.length - 1]);
      return;
    }

    if (!desktop) {
      if (key === 'ArrowLeft' && list && list.classList.contains('sub-menu')) {
        event.preventDefault();
        const parent_item = parent_item_of(list);
        if (parent_item) close_panel(parent_item);
        return;
      }
      if (key === 'ArrowRight' && submenu && is_parent_control) {
        event.preventDefault();
        open_item(item, true);
        return;
      }
      if (key === 'ArrowDown' || key === 'ArrowUp') {
        if (move_in_list(item, key === 'ArrowDown' ? 1 : -1)) event.preventDefault();
      }
      return;
    }

    if (on_top && (key === 'ArrowLeft' || key === 'ArrowRight')) {
      if (move_in_list(item, key === 'ArrowRight' ? 1 : -1)) event.preventDefault();
      return;
    }

    if (on_top && key === 'ArrowDown' && submenu && is_parent_control) {
      event.preventDefault();
      open_item(item, true);
      return;
    }

    if (on_top && key === 'ArrowUp' && item.classList.contains('-active')) {
      event.preventDefault();
      close_panel(item);
      return;
    }

    if (!on_top && (key === 'ArrowDown' || key === 'ArrowUp')) {
      if (move_in_list(item, key === 'ArrowDown' ? 1 : -1)) event.preventDefault();
      return;
    }

    if (!on_top && key === 'ArrowLeft') {
      event.preventDefault();
      const parent_item = parent_item_of(list);
      if (parent_item) close_panel(parent_item);
      return;
    }

    if (!on_top && key === 'ArrowRight' && submenu && is_parent_control) {
      event.preventDefault();
      open_item(item, true);
    }
  }

  function focusable_in(root) {
    const nodes = root.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])');
    const visible = [];
    for (let i = 0; i < nodes.length; i += 1) {
      if (is_operable(nodes[i])) visible.push(nodes[i]);
    }
    return visible;
  }

  function trap_root() {
    if (!dd_main_menu) return null;
    return dd_main_menu.closest('.dd-header') || dd_main_menu;
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
      close_panel(open_item);
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
    ensure_panel_chrome(item, submenu, link);
    if (link && is_dead_link(link)) {
      link.setAttribute('role', 'button');
      link.setAttribute('aria-expanded', 'false');
      link.setAttribute('aria-controls', submenu.id);
    }
    apply_expanded(item, false);

    toggle.addEventListener('click', (event) => {
      event.preventDefault();
      toggle_parent(item, !is_desktop_nav() || event.detail === 0);
    });

    if (link && is_dead_link(link)) {
      link.addEventListener('keydown', (event) => {
        if (event.key !== ' ') return;
        event.preventDefault();
      });
      link.addEventListener('keyup', (event) => {
        if (event.key !== ' ') return;
        event.preventDefault();
        ignore_dead_click = true;
        toggle_parent(item, true);
        window.setTimeout(() => {
          ignore_dead_click = false;
        }, 0);
      });
      link.addEventListener('click', (event) => {
        event.preventDefault();
        if (ignore_dead_click) return;
        toggle_parent(item, !is_desktop_nav() || event.detail === 0);
      });
    }

    if (link) {
      link.addEventListener('pointerenter', () => {
        if (!is_desktop_nav() || !fine_pointer_query.matches) return;
        if (!item.classList.contains('-active')) open_item(item, false);
      });
    }

    item.addEventListener('mouseleave', () => {
      if (!is_desktop_nav() || !fine_pointer_query.matches) return;
      if (!item.contains(document.activeElement)) {
        apply_expanded(item, false);
        sync_inert();
      }
    });

    item.addEventListener('focusout', () => {
      if (!is_desktop_nav()) return;
      window.setTimeout(() => {
        if (!is_desktop_nav()) return;
        if (item.contains(document.activeElement)) return;
        if (!item.classList.contains('-active')) return;
        apply_expanded(item, false);
        sync_inert();
      }, 0);
    });
  }

  function setup_menu(root) {
    const items = root.querySelectorAll('.menu-item');
    for (let i = 0; i < items.length; i += 1) {
      if (direct_submenu(items[i])) setup_parent(items[i]);
    }
    sync_inert();
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
      sync_panel_box();
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
    if (!is_desktop_nav()) return;
    const open_items = document.querySelectorAll('.navigation.-main-menu .menu-item.-active');
    let changed = false;
    for (let i = open_items.length - 1; i >= 0; i -= 1) {
      const item = open_items[i];
      if (item.contains(event.target)) continue;
      if (item.contains(document.activeElement)) {
        const toggle = direct_toggle(item);
        if (toggle) focus_control(toggle);
      }
      apply_expanded(item, false);
      changed = true;
    }
    if (changed) sync_inert();
  });

  function on_breakpoint_change(event) {
    close_submenus(document);
    if (!event.matches || document.querySelector('dialog[open]')) return;
    if (dd_main_menu && dd_main_menu.classList.contains('-active')) {
      dd_navigation_set_open(false);
    }
  }

  if (typeof desktop_query.addEventListener === 'function') {
    desktop_query.addEventListener('change', on_breakpoint_change);
  }

  window.addEventListener('resize', () => {
    if (is_mobile_open()) sync_panel_box();
  });
}

document.addEventListener('DOMContentLoaded', () => {
  dd_navigation();
});
