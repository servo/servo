// META: global=window
// META: timeout=long
// META: title=Fetch API: keepalive handling
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=../resources/keepalive-helper.js

'use strict';

const {
  HTTP_NOTSAMESITE_ORIGIN,
  HTTP_REMOTE_ORIGIN,
  HTTP_REMOTE_ORIGIN_WITH_DIFFERENT_PORT
} = get_host_info();

/**
 * In a different-site iframe, test to fetch a keepalive URL on the specified
 * document event.
 */
function keepaliveSimpleRequestTest(method) {
  for (const evt of ['load', 'unload', 'pagehide']) {
    const desc =
        `[keepalive] simple ${method} request on '${evt}' [no payload]`;
    promise_test(async (test) => {
      const token1 = token();
      const iframe = document.createElement('iframe');
      iframe.src = getKeepAliveIframeUrl(token1, method, {sendOn: evt});
      document.body.appendChild(iframe);
      await iframeLoaded(iframe);
      if (evt != 'load') {
        iframe.remove();
      }

      assertStashedTokenAsync(desc, token1);
    }, `${desc}; setting up`);
  }
}

for (const method of ['GET', 'POST']) {
  keepaliveSimpleRequestTest(method);
}
