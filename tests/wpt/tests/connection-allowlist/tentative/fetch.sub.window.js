// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function fetch_test(origin, expectation) {
  if (expectation === FAILURE) {
    return promise_test(async t => {
      const fetcher = fetch(`${origin}/common/blank-with-cors.html`, { mode: "cors", credential: "omit" });
      return promise_rejects_js(t, TypeError, fetcher);
    }, `Fetch to ${origin} fails.`);
  }

  promise_test(async t => {
    const r = await fetch(`${origin}/common/blank-with-cors.html`, { mode: "cors", credential: "omit" });
    assert_equals(r.status, 200);
  }, `Fetch to ${origin} succeeds.`);
}

const test_cases = [
  // We're loading this page from `http://hosts[][]`, so that origin should
  // succeed, while its subdomains should fail:
  { origin: "http://{{hosts[][]}}" + port, expectation: SUCCESS },
  { origin: "http://{{hosts[][www]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][www1]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][www2]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][天気の良い日]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[][élève]}}" + port, expectation: FAILURE },

  // Cross-site origins should fail as well:
  { origin: "http://{{hosts[alt][]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][www]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][www1]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][www2]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][天気の良い日]}}" + port, expectation: FAILURE },
  { origin: "http://{{hosts[alt][élève]}}" + port, expectation: FAILURE },
];

for (let i = 0; i < test_cases.length; i++) {
  fetch_test(test_cases[i].origin, test_cases[i].expectation);
}
