// META: script=/common/get-host-info.sub.js
// META: script=resources/worker_redirect_test.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);redirects=block` has been set.

// 1. Same-origin worker script redirect:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL because redirects are explicitly blocked.
worker_script_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_ORIGIN, FAILURE,
    'Same-origin dedicated worker main script fetch with redirect fails due to redirects=block.');

// 2. Same-origin subresource fetch from same-origin worker with same-origin
// redirect:
// worker: local scheme (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: same-origin
// This should FAIL.
worker_subresource_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_ORIGIN, FAILURE,
    'Same-origin subresource fetch from dedicated worker with same-origin redirect fails due to redirects=block.');

// 3. Same-origin subresource fetch from same-origin worker with cross-origin
// redirect:
// worker: local scheme (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: cross-origin
// This should FAIL.
worker_subresource_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_REMOTE_ORIGIN, FAILURE,
    'Same-origin subresource fetch from dedicated worker with cross-origin redirect fails due to redirects=block.');
