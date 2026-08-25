// META: script=/common/get-host-info.sub.js
// META: script=resources/navigation_redirect_test.js

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist:
// (response-origin);redirects=block`. Thus, all redirects should fail.

// Redirect from an allowlisted origin (same-origin):
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL, because of the redirects=block portion of the header.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should FAIL, because of the redirects=block portion of the header.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[alt][]}}' + port, FAILURE);

// Initial navigation to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
navigation_redirect_test(
    'http://{{hosts[alt][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);
