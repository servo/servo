// META: title=Top-level navigation tests with cross origin & user activated child frames
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/sandbox-top-navigation-helper.js

'use strict';

/* ------------------------- USER ACTIVATION TESTS ------------------------- */

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "allow-top-navigation-by-user-activation", "");
  await activate(iframe_1);

  await attemptTopNavigation(iframe_1, true);
}, "Allow top with user activation + user activation");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "allow-top-navigation-by-user-activation", "");

  await attemptTopNavigation(iframe_1, false);
}, "allow-top-navigation-by-user-activation set but no sticky activation");

/* ---------------------- CROSS ORIGIN (A -> B) TESTS ---------------------- */

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_REMOTE_ORIGIN", "allow-top-navigation", "");

  await attemptTopNavigation(iframe_1, true);
}, "A cross-origin frame with frame sandbox flags can navigate top");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_REMOTE_ORIGIN", "", "allow-top-navigation");

  await attemptTopNavigation(iframe_1, false);
}, "A cross-origin frame with delivered sandbox flags can not navigate top");
