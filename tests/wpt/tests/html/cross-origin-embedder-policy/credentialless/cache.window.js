// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

// With COEP:credentialless, requesting a resource without credentials MUST NOT
// return a response requested with credentials. This would be a security
// issue, since COEP:credentialless can be used to enable crossOriginIsolation.
//
// The test the behavior of the HTTP cache:
// 1. b.com stores cookie.
// 2. a.com(COEP:unsafe-none): request b.com's resource.
// 3. a.com(COEP:credentialless): request b.com's resource.
//
// The first time, the resource is requested with credentials. The response is
// served with Cache-Control: max-age=31536000. It enters the cache.
// The second time, the resource is requested without credentials. The response
// in the cache must not be returned.

const cookie_key = "coep_cache_key";
const cookie_value = "coep_cache_value";
const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

const GetCookie = (response) => {
 return parseCookies(JSON.parse(response))[cookie_key];
}

// "same_origin" document with COEP:unsafe-none.
const w_coep_none_token = token();
const w_coep_none_url = same_origin + executor_path + coep_none +
  `&uuid=${w_coep_none_token}`
const w_coep_none = window.open(w_coep_none_url);
add_completion_callback(() => w_coep_none.close());

// "same_origin" document with COEP:credentialles.
const w_coep_credentialless_token = token();
const w_coep_credentialless_url = same_origin + executor_path +
  coep_credentialless + `&uuid=${w_coep_credentialless_token}`
const w_coep_credentialless = window.open(w_coep_credentialless_url);
add_completion_callback(() => w_coep_credentialless.close());

const this_token = token();

// A request toward a "cross-origin" cacheable response.
const request_token = token();
const request_url = cacheableShowRequestHeaders(cross_origin, request_token);

promise_setup(async test => {
  await setCookie(cross_origin, cookie_key, cookie_value + cookie_same_site_none);
}, "Set cookie");

// The "same-origin" COEP:unsafe-none document fetchs a "cross-origin"
// resource. The request is sent with credentials.
promise_setup(async test => {
  send(w_coep_none_token, `
    await fetch("${request_url}", {
      mode : "no-cors",
      credentials: "include",
    });
    send("${this_token}", "Resource fetched");
  `);

  assert_equals(await receive(this_token), "Resource fetched");
  assert_equals(await receive(request_token).then(GetCookie), cookie_value);
}, "Cache a response requested with credentials");

// The "same-origin" COEP:credentialless document fetches the same resource
// without credentials. The HTTP cache must not be used. Instead a second
// request must be made without credentials.
promise_test(async test => {
  send(w_coep_credentialless_token, `
    await fetch("${request_url}", {
      mode : "no-cors",
      credentials: "include",
    });
    send("${this_token}", "Resource fetched");
  `);

  assert_equals(await receive(this_token), "Resource fetched");

  test.step_timeout(test.unreached_func("The HTTP cache has been used"), 1500);
  assert_equals(await receive(request_token).then(GetCookie), undefined);
}, "The HTTP cache must not be used");
