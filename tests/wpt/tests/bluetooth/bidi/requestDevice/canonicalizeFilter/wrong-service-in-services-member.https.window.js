// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'Invalid service must reject the promise.';
const expected = new TypeError();

bluetooth_bidi_test(() => {
  let test_promises = Promise.resolve();
  generateRequestDeviceArgsWithServices(['wrong_service']).forEach(args => {
    test_promises = test_promises.then(
        () => assert_promise_rejects_with_message(
            requestDeviceWithTrustedClick(args), expected));
  });
  return test_promises;
}, test_desc);
