// META: script=/loading/early-hints/resources/early-hints-helpers.sub.js

// The browser honors only the first 103 Early Hints response of a navigation;
// subsequent ones are ignored. Serve two 103 responses, each preconnecting to a
// different origin, and verify only the first origin's preconnect is recorded.
test(() => {
  const params = new URLSearchParams();
  params.set("first-preconnect", CROSS_ORIGIN);
  params.set("second-preconnect", SAME_ORIGIN);
  const url =
      "/loading/early-hints/resources/multiple-early-hints-preconnects.h2.py?" +
      params.toString();
  window.location.replace(new URL(url, window.location));
});
