// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc =
    'RequestDeviceOptions should have \'filters\' if \'exclusionFilters\' is present. Reject with TypeError if not.';
const expected = new DOMException(
    'Failed to execute \'requestDevice\' on \'Bluetooth\': ' +
        '\'filters\' member must be present if \'exclusionFilters\' is present.',
    new TypeError());
const test_specs = [
  {exclusionFilters: []},
  {exclusionFilters: [], acceptAllDevices: true},
  {exclusionFilters: [{}]},
  {exclusionFilters: [{}], acceptAllDevices: true},
  {exclusionFilters: [{name: 'Name'}]},
  {exclusionFilters: [{name: 'Name'}], acceptAllDevices: true},
];

bluetooth_test(() => {
  let test_promises = Promise.resolve();
  test_specs.forEach(args => {test_promises = test_promises.then(() => {
                       return assert_promise_rejects_with_message(
                           requestDeviceWithTrustedClick(args), expected)
                     })});
  return test_promises;
}, test_desc);
