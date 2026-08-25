// META: script=/common/get-host-info.sub.js
// META: script=resources/navigation_redirect_test.js
// The following tests assume the policy `Connection-Allowlist:
// (response-origin)` has been set. Redirects from a connection-allowlisted URL
// should be blocked by default.

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist: (response-origin)`.
// Thus, only `http://{{hosts[][]}}` is allowlisted for navigations, and no
// redirects are allowed.

// Redirect from an allowlisted origin (same-origin):
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL because redirects are default-blocked for allowlisted
// navigations.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should FAIL.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[alt][]}}' + port, FAILURE);

// Initial navigation to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
navigation_redirect_test(
    'http://{{hosts[alt][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);
