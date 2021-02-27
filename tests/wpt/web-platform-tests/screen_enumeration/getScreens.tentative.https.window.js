// META: global=window
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
  assert_equals(typeof self.getScreens, 'function');
}, 'getScreens() is present');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'granted');
  const screensInterface = await self.getScreens();
  const screens = screensInterface.screens;
  assert_greater_than(screens.length, 0);
  assert_true(screens.includes(screensInterface.currentScreen));

  assert_equals(typeof screens[0].availWidth, 'number');
  assert_equals(typeof screens[0].availHeight, 'number');
  assert_equals(typeof screens[0].width, 'number');
  assert_equals(typeof screens[0].height, 'number');
  assert_equals(typeof screens[0].colorDepth, 'number');
  assert_equals(typeof screens[0].pixelDepth, 'number');

  assert_equals(typeof screens[0].availLeft, 'number');
  assert_equals(typeof screens[0].availTop, 'number');
  assert_equals(typeof screens[0].left, 'number');
  assert_equals(typeof screens[0].top, 'number');
  assert_equals(typeof screens[0].orientation, 'object');

  assert_equals(typeof screens[0].isExtended, 'boolean');
  assert_equals(typeof screens[0].isPrimary, 'boolean');
  assert_equals(typeof screens[0].isInternal, 'boolean');
  assert_equals(typeof screens[0].devicePixelRatio, 'number');
  assert_equals(typeof screens[0].id, 'string');
  assert_equals(typeof screens[0].pointerTypes, 'object');
  assert_equals(typeof screens[0].label, 'string');
}, 'getScreens() returns at least 1 Screen with permission granted');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'granted');
  assert_greater_than((await self.getScreens()).screens.length, 0);
  await test_driver.set_permission({name: 'window-placement'}, 'denied');
  await promise_rejects_dom(t, 'NotAllowedError', self.getScreens());
}, 'getScreens() rejects the promise with permission denied');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'granted');
  let iframe = document.body.appendChild(document.createElement('iframe'));
  assert_greater_than((await iframe.contentWindow.getScreens()).screens.length, 0);

  let iframeGetScreens;
  let constructor;
  await new Promise(resolve => {
    iframe.contentWindow.onunload = () => {
      // Grab these before the contentWindow is removed.
      iframeGetScreens = iframe.contentWindow.getScreens;
      constructor = iframe.contentWindow.DOMException;
      resolve();
    };
    document.body.removeChild(iframe);
  });
  assert_not_equals(iframeGetScreens, undefined);
  assert_not_equals(constructor, undefined);

  await t.step_wait(() => !iframe.contentWindow, "execution context invalid");
  assert_equals(iframe.contentWindow, null);
  await promise_rejects_dom(t, 'InvalidStateError', constructor, iframeGetScreens());
}, "getScreens() resolves for attached iframe; rejects for detached iframe");

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'granted');
  let iframe = document.body.appendChild(document.createElement('iframe'));
  const screensInterface = await iframe.contentWindow.getScreens();
  assert_greater_than(screensInterface.screens.length, 0);
  assert_equals(screensInterface.currentScreen, screensInterface.screens[0]);
  iframe.remove();
  await t.step_wait(() => !iframe.contentWindow, "execution context invalid");
  assert_equals(iframe.contentWindow, null);
  assert_equals(screensInterface.screens.length, 0);
  assert_equals(screensInterface.currentScreen, null);
}, "Cached Screens interface from detached iframe doesn't crash, behaves okay");
