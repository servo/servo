// META: script=/common/get-host-info.sub.js
// META: script=resources/test-cases.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin "*://*.hosts[alt]:*")` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;

function fetch_test(origin, expectation, mode, credentials) {
  const settings = { mode, credentials };
  if (expectation === FAILURE) {
    return promise_test(async t => {
      const fetcher = fetch(`${origin}/common/blank-with-cors.html`, settings);
      return promise_rejects_js(t, TypeError, fetcher);
    }, `Fetch ${mode}/${credentials} to ${origin} fails.`);
  }

  promise_test(async t => {
    const r = await fetch(`${origin}/common/blank-with-cors.html`, settings);
    if (mode === 'cors' || origin.startsWith(get_host_info().HTTP_ORIGIN)) {
        assert_equals(r.status, 200);
    } else {
        assert_equals(r.status, 0);
        assert_equals(r.type, 'opaque');
    }
  }, `Fetch ${mode}/${credentials} to ${origin} succeeds.`);
}

const test_cases = get_test_cases(port);
const expectations = get_wildcard_expectations();

for (let i = 0; i < test_cases.length; i++) {
  // Test both CORS and No-CORS modes.
  fetch_test(test_cases[i].origin, expectations[i], "cors", "omit");
  fetch_test(test_cases[i].origin, expectations[i], "no-cors", "omit");
}
