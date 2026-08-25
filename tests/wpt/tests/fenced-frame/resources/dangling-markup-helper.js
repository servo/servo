// These are used in tests that rely on URLs containing dangling markup. See
// https://github.com/whatwg/fetch/pull/519.
const kDanglingMarkupSubstrings = [
  "blo\nck<ed",
  "blo\rck<ed",
  "blo\tck<ed",
  "blo<ck\ned",
  "blo<ck\red",
  "blo<ck\ted",
];

function getTimeoutPromise(t) {
  return new Promise(resolve =>
      t.step_timeout(() => resolve("NOT LOADED"), 1500));
}