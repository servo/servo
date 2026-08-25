// META: title=Top-level navigation tests with cross origin & user activated child frames
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
  const iframe_1 = await createNestedIframe(
      main, 'HTTP_REMOTE_ORIGIN', 'allow-top-navigation', '');

  await attemptTopNavigation(iframe_1, true);
}, 'A cross-origin frame with frame sandbox flags can navigate top');
