// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.
// Redirects for fetches allowed through connection allowlists should be blocked by default.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function fetch_keepalive_redirect_test(origin, target_origin, expectation) {
  const target_url = target_origin + "/common/blank-with-cors.html";
  const url = origin + "/common/redirect.py?status=302&location=" + encodeURIComponent(target_url);

  if (expectation === FAILURE) {
    return promise_test(async t => {
      const fetcher = fetch(url, { mode: "cors", credentials: "omit", keepalive: true });
      return promise_rejects_js(t, TypeError, fetcher);
    }, `Fetch keepalive redirect from ${origin} to ${target_origin} fails.`);
  }

  promise_test(async t => {
    const r = await fetch(url, { mode: "cors", credentials: "omit", keepalive: true });
    assert_equals(r.status, 200);
  }, `Fetch keepalive redirect from ${origin} to ${target_origin} succeeds.`);
}

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist: (response-origin)`.
// Thus, only `http://{{hosts[][]}}` is allowlisted for fetches.

// Same-origin redirect:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL because redirects are default-blocked for allowlisted fetches.
fetch_keepalive_redirect_test("http://{{hosts[][]}}" + port, "http://{{hosts[][]}}" + port, FAILURE);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should FAIL.
fetch_keepalive_redirect_test("http://{{hosts[][]}}" + port, "http://{{hosts[alt][]}}" + port, FAILURE);

// Fetch to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
fetch_keepalive_redirect_test("http://{{hosts[alt][]}}" + port, "http://{{hosts[][]}}" + port, FAILURE);
