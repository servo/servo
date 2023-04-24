// META: global=window
// META: title=Fetch API: keepalive handling
// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=../resources/keepalive-helper.js

'use strict';

const {
  HTTP_NOTSAMESITE_ORIGIN,
  HTTP_REMOTE_ORIGIN,
  HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT
} = get_host_info();

for (const method of ['GET', 'POST']) {
  promise_test(async (test) => {
    const token1 = token();
    const iframe = document.createElement('iframe');
    iframe.src = getKeepAliveIframeUrl(token1, method);
    document.body.appendChild(iframe);
    await iframeLoaded(iframe);
    assert_equals(await getTokenFromMessage(), token1);
    iframe.remove();

    assertStashedTokenAsync(`simple ${method} request: no payload`, token1);
  }, `simple ${method} request: no payload; setting up`);
}
