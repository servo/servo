// META: title=Top-level navigation tests with child frames
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/sandbox-top-navigation-helper.js

'use strict';

/* ----------------------- SAME ORIGIN (A -> A) TESTS ----------------------- */

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "", "allow-top-navigation allow-same-origin");

  await attemptTopNavigation(iframe_1, true);
}, "A same-origin frame with delivered sandbox flags can navigate top");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "allow-top-navigation allow-same-origin", "");

  await attemptTopNavigation(iframe_1, true);
}, "A same-origin frame with frame sandbox flags can navigate top");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "", "");

  await attemptTopNavigation(iframe_1, true);
}, "A same-origin unsandboxed frame can navigate top");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN", "",
      "allow-top-navigation allow-top-navigation-by-user-activation allow-same-origin");

  await attemptTopNavigation(iframe_1, true);
}, "A frame with both top navigation delivered sandbox flags uses the less \
    restrictive one");

promise_test(async t => {
  const main = await setupTest();
  const iframe_1 = await createNestedIframe(main,
      "HTTP_ORIGIN",
      "allow-top-navigation allow-top-navigation-by-user-activation", "");

  await attemptTopNavigation(iframe_1, true);
}, "A frame with both top navigation frame sandbox flags uses the less \
    restrictive one");
