function dd_navigation() {

    const dd_menu_toggle = document.querySelector('.dd-menu__toggle');
    const dd_menu_close = document.querySelector('.dd-menu__close');
    const dd_main_menu = document.querySelector('.navigation.-main-menu');

    function dd_navigation_set_open(is_open) {
      if (dd_menu_toggle) {
        dd_menu_toggle.classList.toggle('-active', is_open);
        dd_menu_toggle.setAttribute('aria-expanded', is_open ? 'true' : 'false');
      }
      if (dd_main_menu) {
        dd_main_menu.classList.toggle('-active', is_open);
      }
      // dd_search_close() clears body.-active, so set the scroll lock after that call.
      document.body.classList.toggle('-active', is_open);
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
      // Used if there is a close button as part of the menu when open on mobile
      dd_menu_close.addEventListener('click', () => {
        dd_navigation_set_open(false);
      });
    }

    // Add click event listeners to menu items with children
    const menuItemsWithChildren = document.querySelectorAll('.menu-item-has-children');
    menuItemsWithChildren.forEach(item => {
      item.addEventListener('click', (event) => {
        event.preventDefault();
        if (item.classList.contains('-active')) {
          // If already active, remove it
          item.classList.remove('-active');
          item.removeAttribute('aria-expanded');
        } else {
          // Remove -active class from all other menu items with children
          menuItemsWithChildren.forEach(otherItem => {
            otherItem.classList.remove('-active');
            otherItem.removeAttribute('aria-expanded');
          });
          // Add -active class to the clicked item
          item.classList.add('-active');
          item.setAttribute('aria-expanded', 'true');
        }
        // Set focus to the clicked item for keyboard navigation
        item.focus();
      });
    });

};

// Initialize on initial page load
document.addEventListener('DOMContentLoaded', () => {
  dd_navigation();
});
