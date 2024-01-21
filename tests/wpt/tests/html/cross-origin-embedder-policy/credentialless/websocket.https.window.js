// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

promise_test_parallel(async test => {
  const same_origin = get_host_info().HTTPS_ORIGIN;
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const cookie_key = "coep_credentialless_websocket";
  const cookie_same_origin = "same_origin";
  const cookie_cross_origin = "cross_origin";

  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  // One window with COEP:none. (control)
  const w_control_token = token();
  const w_control_url = same_origin + executor_path +
    coep_none + `&uuid=${w_control_token}`
  const w_control = window.open(w_control_url);
  add_completion_callback(() => w_control.close());

  // One window with COEP:credentialless. (experiment)
  const w_credentialless_token = token();
  const w_credentialless_url = same_origin + executor_path +
    coep_credentialless + `&uuid=${w_credentialless_token}`;
  const w_credentialless = window.open(w_credentialless_url);
  add_completion_callback(() => w_credentialless.close());

  let WebSocketTest = function(
    description, origin,
    expected_cookies_control,
    expected_cookies_credentialless)
  {
    promise_test_parallel(async test => {
      const token_1 = token();
      const token_2 = token();

      const origin_for_websocket = origin.replace("https", "wss");

      send(w_control_token, `
        var ws = new WebSocket("${showRequestHeaders(origin_for_websocket, token_1)}");
      `);

      send(w_credentialless_token, `
        var ws = new WebSocket("${showRequestHeaders(origin_for_websocket, token_2)}");
      `);

      const headers_control = JSON.parse(await receive(token_1));
      const headers_credentialless = JSON.parse(await receive(token_2));

      assert_equals(parseCookies(headers_control)[cookie_key],
        expected_cookies_control,
        "coep:none => ");
      assert_equals(parseCookies(headers_credentialless)[cookie_key],
        expected_cookies_credentialless,
        "coep:credentialless => ");
    }, `WebSocket ${description}`)
  };

  // Same-origin request always contains Cookies:
  WebSocketTest("same-origin",
    same_origin,
    cookie_same_origin,
    cookie_same_origin);

  // Cross-origin request also always contains Cookies:
  WebSocketTest("cross-origin",
    cross_origin,
    cookie_cross_origin,
    cookie_cross_origin);
}, "Main");
