function dd_tabs() {
  const tabsContainers = document.querySelectorAll('.dd-tabs');

  tabsContainers.forEach(container => {
    const tabs = container.querySelectorAll('[role="tab"]');
    const panels = container.querySelectorAll('[role="tabpanel"]');

    const activate = (tab) => {
      const targetId = tab.getAttribute('aria-controls');
      tabs.forEach(t => {
        const on = t === tab;
        t.classList.toggle('-active', on);
        t.setAttribute('aria-selected', on ? 'true' : 'false');
        t.setAttribute('tabindex', on ? '0' : '-1');
      });
      panels.forEach(panel => {
        const on = panel.id === targetId;
        panel.classList.toggle('-active', on);
        if (on) {
          panel.removeAttribute('hidden');
        } else {
          panel.setAttribute('hidden', '');
        }
      });
    };

    tabs.forEach((tab, i) => {
      tab.addEventListener('click', function (e) {
        e.preventDefault();
        activate(this);
      });
      tab.addEventListener('keydown', function (e) {
        let next = null;
        if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
          next = tabs[(i + 1) % tabs.length];
        } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
          next = tabs[(i - 1 + tabs.length) % tabs.length];
        } else if (e.key === 'Home') {
          next = tabs[0];
        } else if (e.key === 'End') {
          next = tabs[tabs.length - 1];
        }
        if (next) {
          e.preventDefault();
          next.focus();
          activate(next);
        }
      });
    });
  });
}

document.addEventListener('DOMContentLoaded', () => {
  dd_tabs();
});
document.body.addEventListener("htmx:afterSettle", function () {
  dd_tabs();
});
