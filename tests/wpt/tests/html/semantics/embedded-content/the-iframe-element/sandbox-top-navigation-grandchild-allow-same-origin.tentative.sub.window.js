// META: title=Top-level navigation tests with frames that try to give themselves top-nav permission
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/sandbox-top-navigation-helper.sub.js

'use strict';

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main, 'HTTP_ORIGIN', '', '');
  const iframe_2 = await createNestedIframe(
      iframe_1, 'HTTP_ORIGIN', '', 'allow-same-origin allow-top-navigation');

  await attemptTopNavigation(iframe_2, true);
}, 'A sandboxed same-origin grandchild with allow-same-origin can \
    give itself top-nav privileges');
