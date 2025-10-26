// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

promise_test(async t => {
  const r = await fetch("/common/blank-with-cors.html", { mode: "cors", credential: "omit" });
  assert_equals(r.status, 200);
}, "Same-origin fetches succeed when `response-origin` is specified.");

promise_test(async t => {
  const fetcher = fetch(get_host_info().HTTPS_REMOTE_ORIGIN + "/common/blank-with-cors.html", { mode: "cors", credential: "omit" });
  return promise_rejects_js(t, TypeError, fetcher);
}, "Cross-origin fetches fail when `response-origin` is specified.");
