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

async_test(async t => {
  let iframe = document.body.appendChild(document.createElement('iframe'));
  assert_equals(typeof await iframe.contentWindow.isMultiScreen(), 'boolean');

  iframe.contentWindow.onunload = t.step_func(async () => {
    // TODO(crbug.com/1106132): This should reject or resolve; not hang.
    // assert_equals(typeof await iframe.contentWindow.isMultiScreen(), 'boolean');

    let iframeIsMultiScreen = iframe.contentWindow.isMultiScreen;
    let constructor = iframe.contentWindow.DOMException;
    assert_not_equals(iframeIsMultiScreen, undefined);
    assert_not_equals(constructor, undefined);

    await t.step_wait(() => !iframe.contentWindow, "execution context invalid");
    assert_equals(iframe.contentWindow, null);
    await promise_rejects_dom(t, 'InvalidStateError', constructor, iframeIsMultiScreen());
    t.done();
  });

  document.body.removeChild(iframe);
}, "isMultiScreen() resolves for attached iframe; rejects for detached iframe");
