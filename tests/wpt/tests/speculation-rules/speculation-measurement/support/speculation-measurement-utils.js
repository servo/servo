// Shared utilities for speculation measurement tests.
//
// Synchronization principle: never wait on a fixed clock duration to
// determine whether a speculative navigation has (or has not) been started.
// Instead:
//   - For positive assertions ("this URL must appear in navigations"), poll
//     `performance.getSpeculations().navigations` until the entry shows up,
//     using `waitForSpeculationMatching`. Polling is state-driven — the
//     test resumes as soon as the state flips.
//   - For negative assertions ("this URL must NOT appear"), use a *barrier*:
//     add a known-eager candidate alongside the candidate under test in the
//     same rule set, wait for the barrier's entry to appear, and only then
//     check the target URL's absence. Because the browser processes a rule
//     set's candidates in a single mojo message, seeing the barrier means
//     everything else in that rule set has also had its chance.

function isSpeculationMeasurementEnabled() {
  return typeof performance.getSpeculations === 'function';
}

// Safety-net timeout for state-based polling. Only hit if the state fails
// to flip (which is a real test failure, not a timing issue) — the poll
// resolves as soon as the state flips.
const SPECULATION_POLL_SAFETY_MS = 30000;

// Interval between polls. Small enough that overhead is negligible.
const SPECULATION_POLL_INTERVAL_MS = 20;

// Generate a fresh, unique URL under the current origin. Callers use this
// so subtests don't cross-contaminate — an entry for a URL prefetched by
// one subtest would otherwise appear spuriously in a later subtest running
// in the same document.
let uniqueUrlCounter = 0;
function uniqueNavigationUrl(tag) {
  uniqueUrlCounter += 1;
  return new URL(
             `/common/blank.html?spec-measurement-${tag}-${uniqueUrlCounter}` +
                 `-${crypto.randomUUID()}`,
             location.href)
      .href;
}

// Poll `performance.getSpeculations().navigations` until an entry matching
// `predicate` appears. `predicate` may be a URL string (exact match) or a
// callback receiving each entry.
//
// Returns the matching entry, or throws on the safety-net timeout.
async function waitForSpeculationMatching(
    t, predicate, {timeoutMs = SPECULATION_POLL_SAFETY_MS} = {}) {
  const match =
      typeof predicate === 'string' ? (n => n.url === predicate) : predicate;
  const deadline = performance.now() + timeoutMs;
  while (true) {
    const entry = performance.getSpeculations().navigations.find(match);
    if (entry)
      return entry;
    if (performance.now() > deadline) {
      throw new Error(
          'Timed out waiting for a matching navigations entry to appear ' +
          '(safety net; this indicates a real failure to enact the ' +
          'speculative navigation).');
    }
    await new Promise(r => t.step_timeout(r, SPECULATION_POLL_INTERVAL_MS));
  }
}

// Barrier-based synchronization for negative assertions.
//
// The browser processes each rule set atomically: all of its candidates are
// consumed together. So if we insert a rule set containing one URL under test
// PLUS an `immediate`-eagerness "barrier" URL, and we then wait until the
// barrier appears in `performance.getSpeculations().navigations`, we know the
// browser has had time to enact anything else it was going to enact from that
// same rule set. If the URL
// under test is still absent, it's absent because the browser chose not to
// enact it — which is exactly what the negative assertion is checking.
//
// We use `immediate` to trigger the barrier immediately, without needing any
// user interaction.
//
// Returns { barrierUrl, ruleSet } so the caller can insert the rule set
// with insertSpeculationRules(ruleSet) and later verify via
// awaitBarrierAndSnapshot(t, barrierUrl).
function makeBarrierRuleSet(otherPrefetchRules, {tag = 'barrier'} = {}) {
  const barrierUrl = uniqueNavigationUrl(tag);
  const ruleSet = {
    prefetch: [
      ...otherPrefetchRules,
      {urls: [barrierUrl], eagerness: 'immediate'},
    ]
  };
  return {barrierUrl, ruleSet};
}

// Wait for the barrier URL to appear in navigations, then return the current
// navigations array. Any URL still absent at this point is absent because
// the browser decided not to enact it, not because of timing.
async function awaitBarrierAndSnapshot(t, barrierUrl) {
  await waitForSpeculationMatching(t, barrierUrl);
  return performance.getSpeculations().navigations;
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

// Generate a unique cross-origin URL to preconnect to. Each call returns a
// distinct origin (via a random subdomain) so tests don't collide. The
// connection itself is fire-and-forget and need not succeed for the renderer
// to record it, so the host does not need to resolve.
let preconnectOriginCounter = 0;
function uniquePreconnectUrl({path = '/', scheme = 'https'} = {}) {
  const host = `host-${++preconnectOriginCounter}-${Date.now()}.preconnect.test`;
  return `${scheme}://${host}${path}`;
}

// Add a <link rel=preconnect>. Preconnect has no load/error event, but the
// renderer records it synchronously while processing the inserted element, so
// the entry is observable as soon as this resolves.
//
// Options:
//   t:           test object (for cleanup)
//   href:        the origin/URL to preconnect to
//   crossorigin: the crossorigin attribute value, or null for none
async function addPreconnect({t, href, crossorigin = null} = {}) {
  const link = document.createElement('link');
  link.rel = 'preconnect';
  link.href = href;
  if (crossorigin !== null) {
    link.crossOrigin = crossorigin;
  }
  document.head.appendChild(link);
  t.add_cleanup(() => link.remove());
  // Recording is synchronous on insertion; yield a frame for robustness.
  await new Promise(resolve => requestAnimationFrame(() => resolve()));
  return {link, href};
}

// Find all PreconnectData entries matching the given serialized origin.
function findPreconnectsByOrigin(preconnects, origin) {
  return preconnects.filter(p => p.origin === origin);
}

// Map crossorigin attribute value to expected CrossOriginMode enum string.
function expectedCrossOriginMode(crossorigin) {
  if (crossorigin === null || crossorigin === undefined) return 'none';
  if (crossorigin === '' || crossorigin === 'anonymous') return 'anonymous';
  if (crossorigin === 'use-credentials') return 'use-credentials';
  return 'none';
}
