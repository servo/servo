// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

promise_test_parallel(async test => {
  const same_origin = get_host_info().HTTPS_ORIGIN;
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const cookie_key = "dip_credentialless_link";
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

  // One window with DIP:credentialless. (experiment)
  const w_credentialless_token = token();
  const w_credentialless_url = same_origin + executor_path +
    dip_credentialless + `&uuid=${w_credentialless_token}`;
  const w_credentialless = window.open(w_credentialless_url);
  add_completion_callback(() => w_credentialless.close());

  let linkTest = function(
    description, origin, mode,
    expected_cookies_control,
    expected_cookies_credentialless)
  {
    promise_test_parallel(async test => {
      const token_1 = token();
      const token_2 = token();

      send(w_control_token, `
        let link = document.createElement("link");
        link.href = "${showRequestHeaders(origin, token_1)}";
        link.rel = "stylesheet";
        ${mode}
        document.head.appendChild(link);
      `);
      send(w_credentialless_token, `
        let link = document.createElement("link");
        link.href = "${showRequestHeaders(origin, token_2)}";
        link.rel = "stylesheet";
        ${mode}
        document.head.appendChild(link);
      `);

      const headers_control = JSON.parse(await receive(token_1));
      const headers_credentialless = JSON.parse(await receive(token_2));

      assert_equals(parseCookies(headers_control)[cookie_key],
        expected_cookies_control,
        "dip:none => ");
      assert_equals(parseCookies(headers_credentialless)[cookie_key],
        expected_cookies_credentialless,
        "dip:credentialless => ");
    }, `link ${description}`)
  };

  // Same-origin request always contains Cookies:
  linkTest("same-origin + undefined",
    same_origin, '',
    cookie_same_origin,
    cookie_same_origin);
  linkTest("same-origin + anonymous",
    same_origin, 'link.crossOrigin="anonymous"',
    cookie_same_origin,
    cookie_same_origin);
  linkTest("same-origin + use-credentials",
    same_origin, 'link.crossOrigin="use-credentials"',
    cookie_same_origin,
    cookie_same_origin);

  // Cross-origin request contains cookies in the following cases:
  // - DIP:credentialless is not set.
  // - link.crossOrigin is `use-credentials`.
  linkTest("cross-origin + undefined",
    cross_origin, '',
    cookie_cross_origin,
    undefined);
  linkTest("cross-origin + anonymous",
    cross_origin, 'link.crossOrigin="anonymous"',
    undefined,
    undefined);
  linkTest("cross-origin + use-credentials",
    cross_origin, 'link.crossOrigin="use-credentials"',
    cookie_cross_origin,
    cookie_cross_origin);
}, "Main");
