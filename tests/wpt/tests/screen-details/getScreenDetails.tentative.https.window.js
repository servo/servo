// META: global=window
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
  assert_equals(typeof self.getScreenDetails, 'function');
}, 'getScreenDetails() is present');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-management'}, 'granted');
  const screenDetails = await self.getScreenDetails();
  assert_true(ScreenDetails.prototype.isPrototypeOf(screenDetails));
  assert_true(ScreenDetailed.prototype.isPrototypeOf(screenDetails.currentScreen));
  assert_true(Array.prototype.isPrototypeOf(screenDetails.screens));
  assert_greater_than(screenDetails.screens.length, 0);
  assert_true(ScreenDetailed.prototype.isPrototypeOf(screenDetails.screens[0]));
  assert_true(screenDetails.screens.includes(screenDetails.currentScreen));
}, 'getScreenDetails() returns the expected interface with permission granted');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-management'}, 'granted');
  assert_greater_than((await self.getScreenDetails()).screens.length, 0);
  await test_driver.set_permission({name: 'window-management'}, 'denied');
  await promise_rejects_dom(t, 'NotAllowedError', self.getScreenDetails());
}, 'getScreenDetails() rejects the promise with permission denied');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-management'}, 'granted');
  let iframe = document.body.appendChild(document.createElement('iframe'));
  assert_greater_than((await iframe.contentWindow.getScreenDetails()).screens.length, 0);

  let iframeGetScreens;
  let constructor;
  await new Promise(resolve => {
    iframe.contentWindow.onunload = () => {
      // Grab these before the contentWindow is removed.
      iframeGetScreens = iframe.contentWindow.getScreenDetails;
      constructor = iframe.contentWindow.DOMException;
      resolve();
    };
    document.body.removeChild(iframe);
  });
  assert_not_equals(iframeGetScreens, undefined);
  assert_not_equals(constructor, undefined);

  await t.step_wait(() => !iframe.contentWindow, 'execution context invalid');
  assert_equals(iframe.contentWindow, null);
  await promise_rejects_dom(t, 'InvalidStateError', constructor, iframeGetScreens());
}, 'getScreenDetails() resolves for attached iframe; rejects for detached iframe');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-management'}, 'granted');
  let iframe = document.body.appendChild(document.createElement('iframe'));
  const screenDetails = await iframe.contentWindow.getScreenDetails();
  assert_greater_than(screenDetails.screens.length, 0);
  assert_equals(screenDetails.currentScreen, screenDetails.screens[0]);
  iframe.remove();
  await t.step_wait(() => !iframe.contentWindow, 'execution context invalid');
  assert_equals(iframe.contentWindow, null);
  assert_equals(screenDetails.screens.length, 0);
  assert_equals(screenDetails.currentScreen, null);
}, 'Cached ScreenDetails interface from detached iframe does not crash, behaves okay');
