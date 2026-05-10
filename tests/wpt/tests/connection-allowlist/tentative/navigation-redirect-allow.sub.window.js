// META: script=/common/get-host-info.sub.js
// META: script=resources/navigation_redirect_test.js

// We're loading this page from `http://{{hosts[][]}}`.
// The connection allowlist header is `Connection-Allowlist:
// (response-origin);redirects=allow`. Thus, all redirects should succeed.

// Redirect from an allowlisted origin (same-origin):
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should SUCCEED, because of the redirects=allow portion of the header.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[][]}}' + port, SUCCESS);

// Redirect from an allowlisted origin to a different origin:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[alt][]}} (not allowed)
// This should SUCCEED, because of the redirects=allow portion of the header,
// regardless of what the target origin is.
navigation_redirect_test(
    'http://{{hosts[][]}}' + port, 'http://{{hosts[alt][]}}' + port, SUCCESS);

// Initial navigation to a non-allowlisted origin:
// origin: http://{{hosts[alt][]}} (not allowed)
// This is blocked before the redirect even happens.
navigation_redirect_test(
    'http://{{hosts[alt][]}}' + port, 'http://{{hosts[][]}}' + port, FAILURE);
