function dd_footer_copywrite() {
  const year = String(new Date().getFullYear());
  const token = '[[YYYY]]';
  document.querySelectorAll('.dd-footer__copyright').forEach((root) => {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    let node = walker.nextNode();
    while (node) {
      if (node.nodeValue && node.nodeValue.includes(token)) {
        node.nodeValue = node.nodeValue.split(token).join(year);
      }
      node = walker.nextNode();
    }
  });
}

// Run once, after the page has fully loaded.
document.addEventListener('DOMContentLoaded', () => {
  dd_footer_copyright();
});
