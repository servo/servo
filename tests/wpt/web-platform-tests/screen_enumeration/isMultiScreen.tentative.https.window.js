// META: global=window
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
  assert_equals(typeof self.isMultiScreen, 'function');
}, 'isMultiScreen() is present');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'granted');
  assert_equals(typeof await self.isMultiScreen(), 'boolean');
}, 'isMultiScreen() returns a boolean value with permission granted');

promise_test(async t => {
  await test_driver.set_permission({name: 'window-placement'}, 'denied');
  assert_equals(typeof await self.isMultiScreen(), 'boolean');
}, 'isMultiScreen() returns a boolean value with permission denied');

promise_test(async t => {
  let iframe = document.body.appendChild(document.createElement('iframe'));
  assert_equals(typeof await iframe.contentWindow.isMultiScreen(), 'boolean');

  let iframeIsMultiScreen;
  let constructor;
  await new Promise(resolve => {
    iframe.contentWindow.onunload = () => {
      // Grab these before the contentWindow is removed.
      iframeIsMultiScreen = iframe.contentWindow.isMultiScreen;
      constructor = iframe.contentWindow.DOMException;
      resolve();
    };
    document.body.removeChild(iframe);
  });


  // TODO(crbug.com/1106132): This should reject or resolve; not hang.
  // assert_equals(typeof await iframe.contentWindow.isMultiScreen(), 'boolean');
  assert_not_equals(iframeIsMultiScreen, undefined);
  assert_not_equals(constructor, undefined);

  await t.step_wait(() => !iframe.contentWindow, "execution context invalid");
  assert_equals(iframe.contentWindow, null);
  await promise_rejects_dom(t, 'InvalidStateError', constructor, iframeIsMultiScreen());
}, "isMultiScreen() resolves for attached iframe; rejects for detached iframe");
