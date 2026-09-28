function dd_search() {

    const dd_search_toggle = document.querySelector('.dd-search__toggle');
    const dd_search_close = document.querySelector('.dd-search__close');
    const dd_search = document.querySelector('.dd-search');

  if (dd_search_toggle) {
    dd_search_toggle.addEventListener('click', () => {
        // Toggle the -active class on the button itself
        dd_search_toggle.classList.toggle('-active');
        // Toggle the -active class on the search div
        dd_search.classList.toggle('-active');
        // Force close menu
        // Toggle the -active class on the menu element
        document.querySelector('.dd-menu__toggle').classList.remove('-active');
        document.querySelector('.navigation.-main-menu').classList.remove('-active');
      });
    }

  if (dd_search_close) {
      // Used if there is a close button as part of the search when open
      dd_search_close.addEventListener('click', () => {
        // Toggle the -active class on the button itself
        dd_search_toggle.classList.toggle('-active');
        // Toggle the -active class on the search div
        dd_search.classList.toggle('-active');
      });
    }

};

// Initialize on initial page load
document.addEventListener('DOMContentLoaded', () => {
  dd_search();
});
