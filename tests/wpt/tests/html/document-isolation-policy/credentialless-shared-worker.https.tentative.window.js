// META: variant=?request_origin=same_origin&worker_dip=none&window_dip=none
// META: variant=?request_origin=same_origin&worker_dip=none&window_dip=credentialless
// META: variant=?request_origin=same_origin&worker_dip=credentialless&window_dip=none
// META: variant=?request_origin=same_origin&worker_dip=credentialless&window_dip=credentialless
// META: variant=?request_origin=cross_origin&worker_dip=none&window_dip=none
// META: variant=?request_origin=cross_origin&worker_dip=none&window_dip=credentialless
// META: variant=?request_origin=cross_origin&worker_dip=credentialless&window_dip=none
// META: variant=?request_origin=cross_origin&worker_dip=credentialless&window_dip=credentialless
// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

// Test description:
//   Request a resource from a SharedWorker. Check the request's cookies.
//
// Variant:
//   - The Window DIP policy: none or credentialless.
//   - The SharedWorker DIP policy: none or credentialless.
//   - The SharedWorker's request URL origin: same-origin or cross-origin.

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = token();
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

const variants = new URLSearchParams(window.location.search);
const window_dip = variants.get('window_dip') == 'none'
  ? dip_none
  : dip_credentialless;
const worker_dip = variants.get('worker_dip') == 'none'
  ? dip_none
  : dip_credentialless;
const request_origin = variants.get('request_origin') == 'same-origin'
  ? same_origin
  : cross_origin;

// When using DIP:credentialless: cross-origin no-cors request do not include
// credentials. Note: This must not depend on the window's DIP policy.
const worker_expected_cookie =
  request_origin == same_origin
  ? cookie_same_origin
  : (worker_dip == dip_credentialless
    ? undefined
    : cookie_cross_origin);

// From a JSON representing the `response` HTTP headers key-values, return the
// cookie corresponding to the `cookie_key`.
const get_cookie = (response) => {
  const headers_credentialless = JSON.parse(response);
  return parseCookies(headers_credentialless)[cookie_key];
}

promise_test(async test => {
  // 0. Populate cookies for the two origins.
  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  // 1. Create the popup with the `window_dip` DIP policy:
  const popup = environments.document(window_dip)[0];

  // 2. Create the worker with the `worker_dip` DIP policy:
  const worker_token = token();
  const worker_error = token();
  const worker_src = same_origin + executor_worker_path + worker_dip +
    `&uuid=${worker_token}`;
  send(popup, `
    let worker = new SharedWorker("${worker_src}", {});
    worker.onerror = () => {
      send("${worker_error}", "Worker blocked");
    }
  `);

  // 3. Request the resource from the worker, with the `request_origin` origin.
  const request_token = token();
  const request_url = showRequestHeaders(request_origin, request_token);
  send(worker_token, `fetch("${request_url}", {
    mode: 'no-cors',
    credentials: 'include',
  })`);
  const request_cookie = await Promise.race([
    receive(worker_error),
    receive(request_token).then(get_cookie)
  ]);

  assert_equals(request_cookie, worker_expected_cookie);
})
