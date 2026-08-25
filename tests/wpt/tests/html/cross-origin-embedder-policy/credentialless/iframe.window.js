// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js
const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = "coep_redirect";
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

// Operate on a window with COEP:credentialless.
const w_token = token();
const w_url = same_origin + executor_path + coep_credentialless +
              `&uuid=${w_token}`
const w = window.open(w_url);
add_completion_callback(() => w.close());

// Check whether COEP:credentialless applies to navigation request. It
// shouldn't.
const iframeTest = function(name, origin, expected_cookies) {
  promise_test_parallel(async test => {
    const token_request = token();
    const url = showRequestHeaders(origin, token_request);

    send(w_token, `
      const iframe = document.createElement("iframe");
      iframe.src = "${url}";
      document.body.appendChild(iframe);
    `);

    const headers = JSON.parse(await receive(token_request));
    assert_equals(parseCookies(headers)[cookie_key], expected_cookies);
  }, name)
};

promise_test_parallel(async test => {
  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin +
      cookie_same_site_none),
    setCookie(cross_origin, cookie_key, cookie_cross_origin +
      cookie_same_site_none),
  ]);

  iframeTest("same-origin", same_origin, cookie_same_origin);
  iframeTest("cross-origin", cross_origin, cookie_cross_origin);
}, "Setup");
