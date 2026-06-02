// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'Services member must contain at least one service.';
const expected = new TypeError();

bluetooth_bidi_test(() => {
  let test_promises = Promise.resolve();
  generateRequestDeviceArgsWithServices([]).forEach(
      args => {
          test_promises = test_promises.then(
              () => assert_promise_rejects_with_message(
                  requestDeviceWithTrustedClick(args), expected,
                  'Services member must contain at least one service'))});
  return test_promises;
}, test_desc);
