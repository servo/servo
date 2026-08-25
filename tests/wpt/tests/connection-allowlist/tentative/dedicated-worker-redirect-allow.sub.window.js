// META: script=/common/get-host-info.sub.js
// META: script=resources/worker_redirect_test.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin);redirects=allow` has been set.

// 1. Same-origin worker script redirect:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should SUCCEED because redirects are unconditionally enabled via
// redirects=allow.
worker_script_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_ORIGIN, SUCCESS,
    'Same-origin dedicated worker main script fetch with redirect succeeds due to redirects=allow.');

// 2. Same-origin subresource fetch from same-origin worker with same-origin
// redirect:
// worker: local scheme (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: same-origin
// This should SUCCEED.
worker_subresource_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_ORIGIN, SUCCESS,
    'Same-origin subresource fetch from dedicated worker with same-origin redirect succeeds due to redirects=allow.');

// 3. Same-origin subresource fetch from same-origin worker with cross-origin
// redirect:
// worker: local scheme (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: cross-origin
// This should SUCCEED.
worker_subresource_redirect_test(
    get_host_info().HTTP_ORIGIN, get_host_info().HTTP_REMOTE_ORIGIN, SUCCESS,
    'Same-origin subresource fetch from dedicated worker with cross-origin redirect succeeds due to redirects=allow.');
