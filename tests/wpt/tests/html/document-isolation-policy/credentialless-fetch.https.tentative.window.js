// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

promise_test(async test => {
  const same_origin = get_host_info().HTTPS_ORIGIN;
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const cookie_key = "dip_credentialless_fetch";
  const cookie_same_origin = "same_origin";
  const cookie_cross_origin = "cross_origin";

  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  // One window with DIP:none. (control)
  const w_control_token = token();
  const w_control_url = same_origin + executor_path +
    dip_none + `&uuid=${w_control_token}`
  const w_control = window.open(w_control_url);
  add_completion_callback(() => w_control.close());

  // One window with DIP:isolate-and-credentialless. (experiment)
  const w_credentialless_token = token();
  const w_credentialless_url = same_origin + executor_path +
    dip_credentialless + `&uuid=${w_credentialless_token}`;
  const w_credentialless = window.open(w_credentialless_url);
  add_completion_callback(() => w_credentialless.close());

  const fetchTest = function(
    description, origin, mode, credentials,
    expected_cookies_control,
    expected_cookies_credentialless)
  {
    promise_test_parallel(async test => {
      const token_1 = token();
      const token_2 = token();

      send(w_control_token, `
        fetch("${showRequestHeaders(origin, token_1)}", {
          mode:"${mode}",
          credentials: "${credentials}",
        });
      `);
      send(w_credentialless_token, `
        fetch("${showRequestHeaders(origin, token_2)}", {
          mode:"${mode}",
          credentials: "${credentials}",
        });
      `);

      const headers_control = JSON.parse(await receive(token_1));
      const headers_credentialless = JSON.parse(await receive(token_2));

      assert_equals(parseCookies(headers_control)[cookie_key],
        expected_cookies_control,
        "dip:none => ");
      assert_equals(parseCookies(headers_credentialless)[cookie_key],
        expected_cookies_credentialless,
        "dip:isolate-and-credentialless => ");
    }, `fetch ${description}`)
  };

  // Cookies are never sent with credentials='omit'
  fetchTest("same-origin + no-cors + credentials:omit",
    same_origin, 'no-cors', 'omit',
    undefined,
    undefined);
  fetchTest("same-origin + cors + credentials:omit",
    same_origin, 'cors', 'omit',
    undefined,
    undefined);
  fetchTest("cross-origin + no-cors + credentials:omit",
    cross_origin, 'no-cors', 'omit',
    undefined,
    undefined);
  fetchTest("cross-origin + cors + credentials:omit",
    cross_origin, 'cors', 'omit',
    undefined,
    undefined);

  // Same-origin request contains Cookies.
  fetchTest("same-origin + no-cors + credentials:include",
    same_origin, 'no-cors', 'include',
    cookie_same_origin,
    cookie_same_origin);
  fetchTest("same-origin + cors + credentials:include",
    same_origin, 'cors', 'include',
    cookie_same_origin,
    cookie_same_origin);
  fetchTest("same-origin + no-cors + credentials:same-origin",
    same_origin, 'no-cors', 'same-origin',
    cookie_same_origin,
    cookie_same_origin);
  fetchTest("same-origin + cors + credentials:same-origin",
    same_origin, 'cors', 'same-origin',
    cookie_same_origin,
    cookie_same_origin);

  // Cross-origin CORS requests contains Cookies, if credentials mode is set to
  // 'include'. This does not depends on DIP.
  fetchTest("cross-origin + cors + credentials:include",
    cross_origin, 'cors', 'include',
    cookie_cross_origin,
    cookie_cross_origin);
  fetchTest("cross-origin + cors + same-origin-credentials",
    cross_origin, 'cors', 'same-origin',
    undefined,
    undefined);

  // Cross-origin no-CORS requests includes Cookies when:
  // 1. credentials mode is 'include'
  // 2. DIP: is not credentialless.
  fetchTest("cross-origin + no-cors + credentials:include",
    cross_origin, 'no-cors', 'include',
    cookie_cross_origin,
    undefined);

  fetchTest("cross-origin + no-cors + credentials:same-origin",
    cross_origin, 'no-cors', 'same-origin',
    undefined,
    undefined);
}, "");
