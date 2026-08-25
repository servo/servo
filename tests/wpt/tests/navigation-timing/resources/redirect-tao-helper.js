// Helpers for navigation redirect-timing TAO tests.
//
// These build a cross-origin server-side redirect chain (served from the "www"
// subdomain, which is cross-origin to the test page) that finally lands back on
// the test page's own origin -- the navigation's "destination origin". Each hop
// is handled by resources/redirect-tao.py, which optionally emits a
// Timing-Allow-Origin header.
//
// See:
//   https://github.com/whatwg/fetch/pull/1931
//   https://github.com/whatwg/html/pull/12513

// The navigation's destination origin (where the chain lands).
const DESTINATION_ORIGIN = location.origin;

// The final, non-redirect document the chain resolves to (same-origin with the
// test page, so its navigation timing entry is readable).
const FINAL_URL =
    make_absolute_url({path: "/navigation-timing/resources/blank-page-green.html"});

// Builds a redirect-chain URL from `hops`, an array with one entry per redirect.
// Each entry is the value to send in that redirect's Timing-Allow-Origin header,
// or null to send no header (i.e. that hop does not opt in).
function redirect_chain_url(hops) {
  let url = FINAL_URL;
  // Build from the last hop backwards, so each redirect points at the next one.
  for (let i = hops.length - 1; i >= 0; i--) {
    const tao = hops[i] === null ? "" : "tao=" + encodeURIComponent(hops[i]) + "&";
    url = make_absolute_url({
      subdomain: "www",
      path: "/navigation-timing/resources/redirect-tao.py",
      query: tao + "location=" + encodeURIComponent(url),
    });
  }
  return url;
}

// Navigates an iframe through the redirect chain described by `hops` and resolves
// with the iframe's PerformanceNavigationTiming entry. `referrerPolicy` is an
// optional referrer policy to apply to the iframe (e.g. "no-referrer").
function navigation_entry_after_redirects(hops, {referrerPolicy} = {}) {
  return new Promise(resolve => {
    const frame = document.createElement("iframe");
    frame.style.cssText = "width: 250px; height: 250px;";
    if (referrerPolicy) {
      frame.referrerPolicy = referrerPolicy;
    }
    frame.onload = () => {
      resolve(frame.contentWindow.performance.getEntriesByType("navigation")[0]);
    };
    frame.src = redirect_chain_url(hops);
    document.body.appendChild(frame);
  });
}

// Asserts that redirect timing is exposed, with `expectedCount` redirects.
function assert_redirect_timing_exposed(entry, expectedCount) {
  assert_equals(entry.type, "navigate", "navigation type");
  assert_equals(entry.redirectCount, expectedCount, "redirectCount");
  assert_greater_than(entry.redirectStart, 0, "redirectStart is exposed");
  assert_greater_than_equal(entry.redirectEnd, entry.redirectStart,
      "redirectEnd is greater than or equal to redirectStart");
}

// Asserts that redirect timing is fully hidden (zeroed out).
function assert_redirect_timing_hidden(entry) {
  assert_equals(entry.type, "navigate", "navigation type");
  assert_equals(entry.redirectCount, 0, "redirectCount is hidden");
  assert_equals(entry.redirectStart, 0, "redirectStart is hidden");
  assert_equals(entry.redirectEnd, 0, "redirectEnd is hidden");
}
