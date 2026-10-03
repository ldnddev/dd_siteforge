const DD_SEARCH_MIN_LENGTH = 2;
const DD_SEARCH_DEBOUNCE_MS = 200;
const DD_SEARCH_OVERLAY_LIMIT = 8;
const DD_SEARCH_WEIGHTS = {
  title: 10,
  headings: 6,
  description: 4,
  path: 3,
  body: 1
};

let dd_search_index_promise = null;
let dd_search_pages = null;
let dd_search_last_focus = null;
let dd_search_overlay_timer = null;
let dd_search_page_timer = null;

function dd_search_normalize(text) {
  return String(text || '')
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/['’]/g, '')
    .replace(/[^a-z0-9]+/g, ' ')
    .trim();
}

function dd_search_tokens(text) {
  const normalized = dd_search_normalize(text);
  return normalized ? normalized.split(/\s+/) : [];
}

function dd_search_escape(text) {
  return String(text || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function dd_search_levenshtein(a, b) {
  if (a === b) {
    return 0;
  }
  if (!a.length) {
    return b.length;
  }
  if (!b.length) {
    return a.length;
  }
  const rows = a.length + 1;
  const cols = b.length + 1;
  const matrix = new Array(rows);
  for (let i = 0; i < rows; i += 1) {
    matrix[i] = new Array(cols);
    matrix[i][0] = i;
  }
  for (let j = 0; j < cols; j += 1) {
    matrix[0][j] = j;
  }
  for (let i = 1; i < rows; i += 1) {
    for (let j = 1; j < cols; j += 1) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,
        matrix[i][j - 1] + 1,
        matrix[i - 1][j - 1] + cost
      );
    }
  }
  return matrix[a.length][b.length];
}

function dd_search_token_match(query_token, field_token) {
  if (query_token === field_token) {
    return 1;
  }
  if (field_token.startsWith(query_token)) {
    return 0.85;
  }
  if (query_token.startsWith(field_token) && field_token.length >= 3) {
    return 0.7;
  }
  if (query_token.length >= 3 && field_token.includes(query_token)) {
    return 0.6;
  }
  const max_distance = query_token.length >= 7 ? 2 : query_token.length >= 4 ? 1 : 0;
  if (!max_distance) {
    return 0;
  }
  if (Math.abs(query_token.length - field_token.length) > max_distance) {
    return 0;
  }
  const distance = dd_search_levenshtein(query_token, field_token);
  if (distance > max_distance) {
    return 0;
  }
  return distance === 1 ? 0.45 : 0.3;
}

function dd_search_best_token_score(query_token, field_tokens) {
  let best = 0;
  for (let i = 0; i < field_tokens.length; i += 1) {
    const score = dd_search_token_match(query_token, field_tokens[i]);
    if (score > best) {
      best = score;
    }
    if (best === 1) {
      break;
    }
  }
  return best;
}

function dd_search_prepare_pages(pages) {
  return (pages || []).map((page) => ({
    url: page.url || '',
    path: page.path || '',
    title: page.title || '',
    description: page.description || '',
    body: page.body || '',
    headings: page.headings || [],
    _title: dd_search_tokens(page.title),
    _headings: dd_search_tokens((page.headings || []).join(' ')),
    _description: dd_search_tokens(page.description),
    _path: dd_search_tokens(`${page.path || ''} ${page.url || ''}`),
    _body: dd_search_tokens(page.body)
  }));
}

function dd_search_index_url() {
  const from_dom = document.documentElement.getAttribute('data-search-index');
  if (from_dom) {
    if (from_dom.indexOf('http') === 0 || from_dom.charAt(0) === '/') {
      return from_dom;
    }
    return new URL(from_dom, `${window.location.origin}/`).pathname;
  }
  return '/search-index.json';
}

function dd_search_load_index() {
  if (dd_search_pages) {
    return Promise.resolve(dd_search_pages);
  }
  if (dd_search_index_promise) {
    return dd_search_index_promise;
  }
  dd_search_index_promise = fetch(dd_search_index_url(), { credentials: 'same-origin' })
    .then((response) => {
      if (!response.ok) {
        throw new Error('Search index could not be loaded.');
      }
      return response.json();
    })
    .then((data) => {
      dd_search_pages = dd_search_prepare_pages(data && data.pages);
      return dd_search_pages;
    })
    .catch((error) => {
      dd_search_index_promise = null;
      throw error;
    });
  return dd_search_index_promise;
}

function dd_search_query(pages, query) {
  const query_tokens = dd_search_tokens(query);
  if (query_tokens.length === 0) {
    return [];
  }
  const results = [];
  for (let i = 0; i < pages.length; i += 1) {
    const page = pages[i];
    let total = 0;
    let matched = true;
    for (let t = 0; t < query_tokens.length; t += 1) {
      const token = query_tokens[t];
      const title_score = dd_search_best_token_score(token, page._title) * DD_SEARCH_WEIGHTS.title;
      const heading_score = dd_search_best_token_score(token, page._headings) * DD_SEARCH_WEIGHTS.headings;
      const description_score = dd_search_best_token_score(token, page._description) * DD_SEARCH_WEIGHTS.description;
      const path_score = dd_search_best_token_score(token, page._path) * DD_SEARCH_WEIGHTS.path;
      const body_score = dd_search_best_token_score(token, page._body) * DD_SEARCH_WEIGHTS.body;
      const token_score = Math.max(title_score, heading_score, description_score, path_score, body_score);
      if (token_score === 0) {
        matched = false;
        break;
      }
      total += token_score;
    }
    if (!matched) {
      continue;
    }
    const title_hits = query_tokens.filter((token) => dd_search_best_token_score(token, page._title) === 1).length;
    if (title_hits === query_tokens.length) {
      total *= 1.25;
    }
    results.push({
      path: page.path || '/',
      title: page.title,
      description: page.description || '',
      score: total
    });
  }
  results.sort((a, b) => b.score - a.score || a.title.localeCompare(b.title));
  return results;
}

function dd_search_set_status(el, text) {
  if (el) {
    el.textContent = text;
  }
}

function dd_search_page_query() {
  const params = new URLSearchParams(window.location.search);
  return params.get('q') || '';
}

function dd_search_results_href(query) {
  return `/search/?q=${encodeURIComponent(query)}`;
}

function dd_search_overlay_el() {
  return document.querySelector('.dd-search');
}

function dd_search_toggle_el() {
  return document.querySelector('.dd-search__toggle');
}

function dd_search_render_list(results_el, results, options) {
  const prefix = options.prefix;
  const limit = options.limit || results.length;
  const shown = results.slice(0, limit);
  let html = '';
  for (let i = 0; i < shown.length; i += 1) {
    const result = shown[i];
    html += `<li class="${prefix}__result">`;
    html += `<a class="${prefix}__result-link" href="${dd_search_escape(result.path)}">`;
    html += `<span class="${prefix}__result-title">${dd_search_escape(result.title)}</span>`;
    if (result.description) {
      html += `<span class="${prefix}__result-description">${dd_search_escape(result.description)}</span>`;
    }
    html += '</a></li>';
  }
  if (results_el) {
    results_el.innerHTML = html;
    if (shown.length) {
      results_el.removeAttribute('hidden');
    } else {
      results_el.setAttribute('hidden', '');
    }
  }
  const more_el = options.more_el;
  if (more_el) {
    if (options.more_href && results.length > shown.length) {
      more_el.innerHTML = `<a class="${prefix}__more-link" href="${dd_search_escape(options.more_href)}">View all ${results.length} results</a>`;
      more_el.removeAttribute('hidden');
    } else {
      more_el.innerHTML = '';
      more_el.setAttribute('hidden', '');
    }
  }
}

function dd_search_set_invalid(input, is_invalid) {
  if (!input) {
    return;
  }
  if (is_invalid) {
    input.setAttribute('aria-invalid', 'true');
  } else {
    input.removeAttribute('aria-invalid');
  }
}

function dd_search_focus_result(link) {
  if (!link) {
    return;
  }
  link.focus();
  if (typeof link.scrollIntoView === 'function') {
    link.scrollIntoView({ block: 'nearest' });
  }
}

function dd_search_open() {
  const overlay = dd_search_overlay_el();
  const toggle = dd_search_toggle_el();
  const input = overlay ? overlay.querySelector('.dd-search__input') : null;
  const menu_toggle = document.querySelector('.dd-menu__toggle');
  const menu = document.querySelector('.navigation.-main-menu');
  if (!overlay || typeof overlay.showModal !== 'function') {
    return;
  }
  dd_search_last_focus = document.activeElement;
  if (!overlay.open) {
    overlay.showModal();
  }
  document.body.classList.add('-active-search');
  if (toggle) {
    toggle.classList.add('-active');
    toggle.setAttribute('aria-expanded', 'true');
  }
  if (menu_toggle) {
    menu_toggle.classList.remove('-active');
  }
  if (menu) {
    menu.classList.remove('-active');
  }
  dd_search_load_index().catch(() => {});
  if (input) {
    input.focus();
    input.select();
  }
}

function dd_search_close() {
  const overlay = dd_search_overlay_el();
  if (overlay && overlay.open) {
    overlay.close();
  }
}

function dd_search_on_closed() {
  const overlay = dd_search_overlay_el();
  const toggle = dd_search_toggle_el();
  const input = overlay ? overlay.querySelector('.dd-search__input') : null;
  document.body.classList.remove('-active-search');
  if (toggle) {
    toggle.classList.remove('-active');
    toggle.setAttribute('aria-expanded', 'false');
  }
  dd_search_set_invalid(input, false);
  const restore = dd_search_last_focus && typeof dd_search_last_focus.focus === 'function'
    ? dd_search_last_focus
    : toggle;
  dd_search_last_focus = null;
  if (restore) {
    restore.focus();
  }
}

function dd_search_run_overlay(query) {
  const overlay = dd_search_overlay_el();
  if (!overlay) {
    return;
  }
  const results_el = overlay.querySelector('.dd-search__results');
  const status_el = overlay.querySelector('.dd-search__status');
  const more_el = overlay.querySelector('.dd-search__more');
  const input = overlay.querySelector('.dd-search__input');
  const trimmed = String(query || '').trim();
  if (!trimmed) {
    dd_search_set_invalid(input, false);
    dd_search_render_list(results_el, [], { prefix: 'dd-search', more_el });
    dd_search_set_status(status_el, '');
    return;
  }
  if (trimmed.length < DD_SEARCH_MIN_LENGTH) {
    dd_search_render_list(results_el, [], { prefix: 'dd-search', more_el });
    dd_search_set_status(status_el, 'Type at least 2 characters.');
    return;
  }
  dd_search_set_invalid(input, false);
  dd_search_set_status(status_el, 'Searching…');
  dd_search_load_index()
    .then((pages) => {
      const results = dd_search_query(pages, trimmed);
      dd_search_render_list(results_el, results, {
        prefix: 'dd-search',
        limit: DD_SEARCH_OVERLAY_LIMIT,
        more_el,
        more_href: results.length > DD_SEARCH_OVERLAY_LIMIT ? dd_search_results_href(trimmed) : ''
      });
      if (!results.length) {
        dd_search_set_status(status_el, `No results for “${trimmed}”.`);
        return;
      }
      const shown = Math.min(results.length, DD_SEARCH_OVERLAY_LIMIT);
      const extra = results.length > shown ? ` Showing ${shown}.` : '';
      dd_search_set_status(status_el, `${results.length} result${results.length === 1 ? '' : 's'} for “${trimmed}”.${extra}`);
    })
    .catch(() => {
      dd_search_render_list(results_el, [], { prefix: 'dd-search', more_el });
      dd_search_set_status(status_el, 'Search is unavailable. Try again later.');
    });
}

function dd_search_sync_page_heading(query) {
  const heading = document.querySelector('.dd-search-page__title');
  const trimmed = String(query || '').trim();
  const title = trimmed ? `Search results for “${trimmed}”` : 'Search';
  if (heading) {
    heading.textContent = title;
  }
  if (document.querySelector('.dd-search-page')) {
    document.title = `${title} | ldnddev`;
  }
}

function dd_search_run_page(query) {
  const page = document.querySelector('.dd-search-page');
  if (!page) {
    return;
  }
  const results_el = page.querySelector('.dd-search-page__results');
  const status_el = page.querySelector('.dd-search-page__status');
  const input = page.querySelector('.dd-search-page__input');
  const trimmed = String(query || '').trim();
  if (input && input.value !== query) {
    input.value = query;
  }
  dd_search_sync_page_heading(trimmed);
  if (!trimmed) {
    dd_search_set_invalid(input, false);
    dd_search_render_list(results_el, [], { prefix: 'dd-search-page' });
    dd_search_set_status(status_el, 'Enter a search term to find pages on this site.');
    return;
  }
  if (trimmed.length < DD_SEARCH_MIN_LENGTH) {
    dd_search_render_list(results_el, [], { prefix: 'dd-search-page' });
    dd_search_set_status(status_el, 'Type at least 2 characters.');
    return;
  }
  dd_search_set_invalid(input, false);
  dd_search_set_status(status_el, 'Searching…');
  dd_search_load_index()
    .then((pages) => {
      const results = dd_search_query(pages, trimmed);
      dd_search_render_list(results_el, results, {
        prefix: 'dd-search-page',
        limit: results.length
      });
      if (!results.length) {
        dd_search_set_status(status_el, `No results for “${trimmed}”.`);
        return;
      }
      dd_search_set_status(status_el, `${results.length} result${results.length === 1 ? '' : 's'} for “${trimmed}”.`);
    })
    .catch(() => {
      dd_search_render_list(results_el, [], { prefix: 'dd-search-page' });
      dd_search_set_status(status_el, 'Search is unavailable. Try again later.');
    });
}

function dd_search_update_page_url(query) {
  const url = new URL(window.location.href);
  const trimmed = String(query || '').trim();
  if (trimmed) {
    url.searchParams.set('q', trimmed);
  } else {
    url.searchParams.delete('q');
  }
  window.history.replaceState({}, '', url);
}

function dd_search_bind_overlay() {
  const overlay = dd_search_overlay_el();
  const toggle = dd_search_toggle_el();
  if (!overlay || overlay.getAttribute('data-search-initialized') === 'true') {
    return;
  }
  overlay.setAttribute('data-search-initialized', 'true');
  if (toggle) {
    toggle.setAttribute('aria-expanded', overlay.open ? 'true' : 'false');
    toggle.setAttribute('aria-controls', overlay.id || 'dd-search');
    toggle.setAttribute('aria-haspopup', 'dialog');
  }

  const close_btn = overlay.querySelector('.dd-search__close');
  const form = overlay.querySelector('.dd-search__form');
  const input = overlay.querySelector('.dd-search__input');
  const results_el = overlay.querySelector('.dd-search__results');

  if (toggle) {
    toggle.addEventListener('click', () => {
      if (overlay.open) {
        dd_search_close();
      } else {
        dd_search_open();
      }
    });
  }
  if (close_btn) {
    close_btn.addEventListener('click', () => {
      dd_search_close();
    });
  }
  overlay.addEventListener('click', (event) => {
    if (event.target === overlay) {
      dd_search_close();
    }
  });
  overlay.addEventListener('close', () => {
    dd_search_on_closed();
  });
  if (form) {
    form.addEventListener('submit', (event) => {
      const value = input ? input.value.trim() : '';
      if (value.length < DD_SEARCH_MIN_LENGTH) {
        event.preventDefault();
        dd_search_set_invalid(input, true);
        dd_search_run_overlay(value);
      }
    });
  }
  if (input) {
    input.addEventListener('input', () => {
      window.clearTimeout(dd_search_overlay_timer);
      dd_search_overlay_timer = window.setTimeout(() => {
        dd_search_run_overlay(input.value);
      }, DD_SEARCH_DEBOUNCE_MS);
    });
    input.addEventListener('keydown', (event) => {
      if (event.key !== 'ArrowDown') {
        return;
      }
      const links = results_el ? results_el.querySelectorAll('.dd-search__result-link') : [];
      if (!links.length) {
        return;
      }
      event.preventDefault();
      dd_search_focus_result(links[0]);
    });
  }
  if (results_el) {
    results_el.addEventListener('keydown', (event) => {
      const links = Array.prototype.slice.call(results_el.querySelectorAll('.dd-search__result-link'));
      const index = links.indexOf(document.activeElement);
      if (index < 0) {
        return;
      }
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        dd_search_focus_result(links[Math.min(index + 1, links.length - 1)]);
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        if (index <= 0 && input) {
          input.focus();
        } else {
          dd_search_focus_result(links[index - 1]);
        }
      }
    });
  }
}

function dd_search_bind_page() {
  const page = document.querySelector('.dd-search-page');
  if (!page || page.getAttribute('data-search-page-initialized') === 'true') {
    return;
  }
  page.setAttribute('data-search-page-initialized', 'true');
  const form = page.querySelector('.dd-search-page__form');
  const input = page.querySelector('.dd-search-page__input');
  const initial = dd_search_page_query();
  if (input) {
    input.value = initial;
  }
  dd_search_run_page(initial);
  if (form) {
    form.addEventListener('submit', (event) => {
      event.preventDefault();
      const value = input ? input.value.trim() : '';
      if (value.length < DD_SEARCH_MIN_LENGTH) {
        dd_search_set_invalid(input, true);
        dd_search_run_page(value);
        return;
      }
      dd_search_update_page_url(value);
      dd_search_run_page(value);
    });
  }
  if (input) {
    input.addEventListener('input', () => {
      window.clearTimeout(dd_search_page_timer);
      dd_search_page_timer = window.setTimeout(() => {
        dd_search_update_page_url(input.value);
        dd_search_run_page(input.value);
      }, DD_SEARCH_DEBOUNCE_MS);
    });
  }
}

function dd_search() {
  dd_search_bind_overlay();
  dd_search_bind_page();
}

document.addEventListener('DOMContentLoaded', () => {
  dd_search();
});
