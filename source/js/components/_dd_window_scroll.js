function dd_window_scroll() {
  /**
   * Window scroll trigger. Bind to scrollEnd to use.
   * example. window.addEventListener('scrollEnd', function () {});
   **/
  window.addEventListener('scroll', function () {
    if (window.scrollTO) clearTimeout(window.scrollTO);
    window.scrollTO = setTimeout(function () {
      window.dispatchEvent(new Event('scrollEnd'));
    }, 0);
  });
}

// Initialize on initial page load
document.addEventListener('DOMContentLoaded', () => {
  dd_window_scroll();
});
