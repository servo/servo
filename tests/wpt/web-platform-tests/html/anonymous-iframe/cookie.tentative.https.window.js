// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = "anonymous_iframe_load_cookie";
const cookie_same_origin = "same_origin";
const cookie_cross_origin = "cross_origin";

const cookieFromResource = async resource_token => {
  let headers = JSON.parse(await receive(resource_token));
  return parseCookies(headers)[cookie_key];
};

// Load an anonymous iframe, return the HTTP request cookies.
const cookieFromAnonymousIframeRequest = async (iframe_origin) => {
  const resource_token = token();
  let iframe = document.createElement("iframe");
  iframe.src = `${showRequestHeaders(iframe_origin, resource_token)}`;
  iframe.anonymous = true;
  document.body.appendChild(iframe);
  return await cookieFromResource(resource_token);
};

// Load a resource `type` from the iframe with `document_token`,
// return the HTTP request cookies.
const cookieFromResourceInIframe =
    async (document_token, resource_origin, type = "img") => {
  const resource_token = token();
  send(document_token, `
    let el = document.createElement("${type}");
    el.src = "${showRequestHeaders(resource_origin, resource_token)}";
    document.body.appendChild(el);
  `);
  return await cookieFromResource(resource_token);
};

promise_test_parallel(async test => {
  await Promise.all([
    setCookie(same_origin, cookie_key, cookie_same_origin),
    setCookie(cross_origin, cookie_key, cookie_cross_origin),
  ]);

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromAnonymousIframeRequest(same_origin),
      undefined
    );
  }, "Anonymous same-origin iframe is loaded without credentials");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromAnonymousIframeRequest(cross_origin),
      undefined
    );
  }, "Anonymous cross-origin iframe is loaded without credentials");

  let iframe_same_origin = newAnonymousIframe(same_origin);
  let iframe_cross_origin = newAnonymousIframe(cross_origin);

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_same_origin, same_origin),
      undefined
    );
  }, "same_origin anonymous iframe can't send same_origin credentials");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_same_origin, cross_origin),
      undefined
    );
  }, "same_origin anonymous iframe can't send cross_origin credentials");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_cross_origin, cross_origin),
      undefined
    );
  }, "cross_origin anonymous iframe can't send cross_origin credentials");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_cross_origin, same_origin),
      undefined
    );
  }, "cross_origin anonymous iframe can't send same_origin credentials");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_same_origin, same_origin,
                                       "iframe"),
      undefined
    );
  }, "same_origin anonymous iframe can't send same_origin credentials "
                        + "on child iframe");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_same_origin, cross_origin,
                                       "iframe"),
      undefined
    );
  }, "same_origin anonymous iframe can't send cross_origin credentials "
                        + "on child iframe");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_cross_origin, cross_origin,
                                       "iframe"),
      undefined
    );
  }, "cross_origin anonymous iframe can't send cross_origin credentials "
                        + "on child iframe");

  promise_test_parallel(async test => {
    assert_equals(
      await cookieFromResourceInIframe(iframe_cross_origin, same_origin,
                                       "iframe"),
      undefined
    );
  }, "cross_origin anonymous iframe can't send same_origin credentials "
                        + "on child iframe");

}, "Setup")
