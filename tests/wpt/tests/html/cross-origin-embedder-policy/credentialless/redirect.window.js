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

// Operate on a window with COEP:credentialless.:
const w_token = token();
const w_url = same_origin + executor_path + coep_credentialless +
              `&uuid=${w_token}`
const w = window.open(w_url);
add_completion_callback(() => w.close());

let redirectTest = function(name,
                            redirect_origin,
                            final_origin,
                            expected_cookies) {
  promise_test_parallel(async test => {
    const token_request = token();
    const url = redirect_origin + "/common/redirect.py?location=" +
      encodeURIComponent(showRequestHeaders(final_origin, token_request));

    send(w_token, `
      const img = document.createElement("img");
      img.src = "${url}";
      document.body.appendChild(img);
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

  redirectTest("same-origin -> same-origin",
    same_origin, same_origin, cookie_same_origin);
  redirectTest("same-origin -> cross-origin",
    same_origin, cross_origin, undefined)
  redirectTest("cross-origin -> same-origin",
    cross_origin, same_origin, undefined);
  redirectTest("cross-origin -> cross-origin",
    cross_origin, cross_origin, undefined);
}, "Setup");
