// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=./resources/common.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = "credentialless_service_worker";
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

promise_test(async t => {
  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  // One iframe with COEP:none. (control)
  const w_control_token = token();
  const w_control_url = same_origin + executor_path +
    coep_none + `&uuid=${w_control_token}`
  const w_control = document.createElement("iframe");
  w_control.src = w_control_url;
  document.body.appendChild(w_control);

  // One iframe with COEP:credentialless. (experiment)
  const w_credentialless_token = token();
  const w_credentialless_url = same_origin + executor_path +
    coep_credentialless + `&uuid=${w_credentialless_token}`;
  const w_credentialless = document.createElement("iframe");
  w_credentialless.src = w_credentialless_url;
  document.body.appendChild(w_credentialless);

  const serviceWorkerTest = function(
    description, origin, coep_for_worker,
    expected_cookies_control,
    expected_cookies_credentialless)
  {
    promise_test(async test => {
      // Create workers for both window.
      const control_worker_token = token();
      const credentialless_worker_token = token();

      const w_control_worker_src = same_origin + executor_worker_path +
        coep_for_worker + `&uuid=${control_worker_token}`;
      const w_control_worker_reg =
        await service_worker_unregister_and_register(
          test, w_control_worker_src, w_control_url);

      const w_credentialless_worker_src = same_origin + executor_worker_path +
        coep_for_worker + `&uuid=${credentialless_worker_token}`;
      const w_credentialless_worker_reg =
        await service_worker_unregister_and_register(
          test, w_credentialless_worker_src, w_credentialless_url);

      // Fetch resources from the workers.
      const control_request_token = token();
      const credentialless_request_token = token();
      const control_request_url = showRequestHeaders(origin, control_request_token);
      const credentialless_request_url = showRequestHeaders(origin, credentialless_request_token);
      send(control_worker_token, `
        fetch("${control_request_url}", {
          mode: 'no-cors',
          credentials: 'include'
        })
      `);
      send(credentialless_worker_token, `
        fetch("${credentialless_request_url}", {
          mode: 'no-cors',
          credentials: 'include'
        })
      `);

      // Retrieve the resource request headers.
      const headers_control = JSON.parse(await receive(control_request_token));
      const headers_credentialless = JSON.parse(await receive(credentialless_request_token));

      assert_equals(parseCookies(headers_control)[cookie_key],
        expected_cookies_control,
        "coep:none => ");
      assert_equals(parseCookies(headers_credentialless)[cookie_key],
        expected_cookies_credentialless,
        "coep:credentialless => ");

      w_control_worker_reg.unregister();
      w_credentialless_worker_reg.unregister();
    }, `fetch ${description}`)
  };

  serviceWorkerTest("same-origin",
    same_origin, coep_none,
    cookie_same_origin,
    cookie_same_origin);

  serviceWorkerTest("same-origin + credentialless worker",
    same_origin, coep_credentialless,
    cookie_same_origin,
    cookie_same_origin);

  serviceWorkerTest("cross-origin",
    cross_origin, coep_none,
    cookie_cross_origin,
    cookie_cross_origin);

  serviceWorkerTest("cross-origin + credentialless worker",
    cross_origin, coep_credentialless,
    undefined,
    undefined);
})
