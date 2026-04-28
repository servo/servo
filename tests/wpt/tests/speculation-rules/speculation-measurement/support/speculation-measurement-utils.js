// Shared utilities for speculation measurement tests.

function isSpeculationMeasurementEnabled() {
  return typeof performance.getSpeculations === 'function';
}

// Static support files for each "as" type, keyed by a unique suffix to allow
// multiple preloads of the same type in one test page without cache collisions.
function supportFileUrl(as, suffix = '') {
  const files = {
    'script': 'support/preloaded-script.js',
    'style':  'support/preloaded-style.css',
    'fetch':  'support/preloaded-data.txt',
  };
  const base = files[as];
  if (!base) throw new Error(`No support file for as="${as}"`);
  // Append a cache-busting query to avoid collisions between tests.
  return new URL(`${base}?v=${suffix || crypto.randomUUID()}`,
                 location.href).href;
}

// Add a <link rel=preload> and wait for it to finish loading.
// Returns {link, href} once the preload fires onload/onerror.
//
// Options:
//   t:           test object (for cleanup)
//   as:          the "as" attribute value (default: "script")
//   crossorigin: the crossorigin attribute value, or null for none
//   href:        explicit URL (auto-generated from support files if omitted)
async function addPreloadAndWait({t, as = 'script', crossorigin = null,
                                  href = null} = {}) {
  if (!href) {
    href = supportFileUrl(as);
  }

  const link = document.createElement('link');
  link.rel = 'preload';
  link.as = as;
  link.href = href;
  if (crossorigin !== null) {
    link.crossOrigin = crossorigin;
  }

  const loaded = new Promise((resolve, reject) => {
    link.onload = resolve;
    link.onerror = resolve;  // still tracked even on error
  });
  document.head.appendChild(link);
  t.add_cleanup(() => link.remove());

  await loaded;
  return {link, href: link.href};
}

// Use a previously-preloaded resource by inserting the appropriate element.
// Returns a promise that resolves when the resource is loaded.
async function usePreload({t, as, href, crossorigin = null} = {}) {
  let el;
  if (as === 'script') {
    el = document.createElement('script');
    el.src = href;
    if (crossorigin !== null) {
      el.crossOrigin = crossorigin;
    }
  } else if (as === 'style') {
    el = document.createElement('link');
    el.rel = 'stylesheet';
    el.href = href;
    if (crossorigin !== null) {
      el.crossOrigin = crossorigin;
    }
  } else if (as === 'fetch') {
    // For fetch-type preloads, use fetch() API to consume.
    const opts = {};
    if (crossorigin === 'anonymous') {
      opts.mode = 'cors';
      opts.credentials = 'same-origin';
    } else if (crossorigin === 'use-credentials') {
      opts.mode = 'cors';
      opts.credentials = 'include';
    }
    await fetch(href, opts);
    return;  // no element to clean up
  } else {
    throw new Error(`usePreload does not support as="${as}" yet`);
  }

  const loaded = new Promise((resolve, reject) => {
    el.onload = resolve;
    el.onerror = resolve;  // still counts as "used" even on error
  });
  document.body.appendChild(el);
  t.add_cleanup(() => el.remove());
  await loaded;
}

// Find a PreloadData entry whose URL contains the given substring.
function findPreloadByUrl(preloads, urlSubstring) {
  return preloads.find(p => p.url.includes(urlSubstring));
}

// Map crossorigin attribute value to expected CrossOriginMode enum string.
function expectedCrossOriginMode(crossorigin) {
  if (crossorigin === null || crossorigin === undefined) return 'none';
  if (crossorigin === '' || crossorigin === 'anonymous') return 'anonymous';
  if (crossorigin === 'use-credentials') return 'use-credentials';
  return 'none';
}
