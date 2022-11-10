// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

promise_test_parallel(async test => {
  const same_origin = get_host_info().HTTPS_ORIGIN;
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const cookie_key = "coep_credentialless_image";
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

  let imgTest = function(
    description, origin, mode,
    expected_cookies_control,
    expected_cookies_credentialless)
  {
    promise_test_parallel(async test => {
      const token_1 = token();
      const token_2 = token();

      send(w_control_token, `
        let img = document.createElement("img");
        img.src = "${showRequestHeaders(origin, token_1)}";
        ${mode};
        document.body.appendChild(img);
      `);
      send(w_credentialless_token, `
        let img = document.createElement("img");
        img.src = "${showRequestHeaders(origin, token_2)}";
        ${mode};
        document.body.appendChild(img);
      `);

      const headers_control = JSON.parse(await receive(token_1));
      const headers_credentialless = JSON.parse(await receive(token_2));

      assert_equals(parseCookies(headers_control)[cookie_key],
        expected_cookies_control,
        "coep:none => ");
      assert_equals(parseCookies(headers_credentialless)[cookie_key],
        expected_cookies_credentialless,
        "coep:credentialless => ");
    }, `image ${description}`)
  };

  // Same-origin request always contains Cookies:
  imgTest("same-origin + undefined",
    same_origin, '',
    cookie_same_origin,
    cookie_same_origin);
  imgTest("same-origin + anonymous",
    same_origin, 'img.crossOrigin="anonymous"',
    cookie_same_origin,
    cookie_same_origin);
  imgTest("same-origin + use-credentials",
    same_origin, 'img.crossOrigin="use-credentials"',
    cookie_same_origin,
    cookie_same_origin);

  // Cross-origin request contains cookies in the following cases:
  // - COEP:credentialless is not set.
  // - img.crossOrigin is `use-credentials`.
  imgTest("cross-origin + undefined",
    cross_origin, '',
    cookie_cross_origin,
    undefined);
  imgTest("cross-origin + anonymous",
    cross_origin, 'img.crossOrigin="anonymous"',
    undefined,
    undefined);
  imgTest("cross-origin + use-credentials",
    cross_origin, 'img.crossOrigin="use-credentials"',
    cookie_cross_origin,
    cookie_cross_origin);
}, "Main");
