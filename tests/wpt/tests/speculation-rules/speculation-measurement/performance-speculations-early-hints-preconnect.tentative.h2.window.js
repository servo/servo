// META: script=/loading/early-hints/resources/early-hints-helpers.sub.js

// Early Hints preconnects with various crossorigin combinations. Each case is
// distinct by (origin, credentials) so they appear as separate entries:
//   - CROSS_ORIGIN, no crossorigin      -> credentials=true
//   - CROSS_ORIGIN, crossorigin=anon    -> credentials=false
//   - SAME_ORIGIN,  use-credentials     -> credentials=true
const testCases = [
  {origin: CROSS_ORIGIN},
  {origin: CROSS_ORIGIN, crossorigin_attr: 'anonymous'},
  {origin: SAME_ORIGIN,  crossorigin_attr: 'use-credentials'},
];

test(() => {
  const preconnects = testCases.map(tc => {
    const entry = {url: tc.origin};
    if ('crossorigin_attr' in tc) {
      entry.crossorigin_attr = tc.crossorigin_attr;
    }
    return entry;
  });
  navigateToTestWithEarlyHintsPreconnects(
      "resources/speculation-measurement-early-hints-preconnect.html",
      preconnects);
});
