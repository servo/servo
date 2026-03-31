// META: script=/common/get-host-info.sub.js
// META: script=resources/fetch_redirect_test.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);redirects=allow` has been set. Redirects for fetches
// allowed through connection allowlists should be allowed.

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist:
// (response-origin);redirects=allow`. Thus, only `http://{{hosts[][]}}` is
// allowlisted for fetches.

// Same-origin redirect:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should SUCCEED, because of the "allow" header param.
fetch_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[][]}}' + port, SUCCESS);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should SUCCEED, because of the "allow" header param.
fetch_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[alt][]}}' + port, SUCCESS);

// Fetch to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
fetch_redirect_test(
    'http://{{hosts[alt][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);
