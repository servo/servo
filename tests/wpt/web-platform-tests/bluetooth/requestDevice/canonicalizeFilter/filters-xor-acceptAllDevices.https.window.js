// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'RequestDeviceOptions should have exactly one of ' +
    '\'filters\' or \'acceptAllDevices:true\'. Reject with TypeError if not.';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on \'Bluetooth\': ' +
        'Either \'filters\' should be present or ' +
        '\'acceptAllDevices\' should be true, but not both.',
    new TypeError());
const test_specs = [
  {}, {optionalServices: ['heart_rate']}, {filters: [], acceptAllDevices: true},
  {filters: [], acceptAllDevices: true, optionalServices: ['heart_rate']}
];

bluetooth_test(() => {
  let test_promises = Promise.resolve();
  test_specs.forEach(
      args => {
          test_promises = test_promises.then(
              () => assert_promise_rejects_with_message(
                  requestDeviceWithTrustedClick(args), expected))});
  return test_promises;
}, test_desc);
