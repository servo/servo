// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'requestDevice with empty namePrefix. ' +
    'Should reject with TypeError.';
const expected = new TypeError();
const test_specs = [
  {filters: [{namePrefix: ''}]}, {filters: [{namePrefix: '', name: 'Name'}]},
  {filters: [{namePrefix: '', services: ['heart_rate']}]},
  {filters: [{namePrefix: '', name: 'Name', services: ['heart_rate']}]},
  {filters: [{namePrefix: ''}], optionalServices: ['heart_rate']},
  {filters: [{namePrefix: '', name: 'Name'}], optionalServices: ['heart_rate']},
  {
    filters: [{namePrefix: '', services: ['heart_rate']}],
    optionalServices: ['heart_rate']
  },
  {
    filters: [{namePrefix: '', name: 'Name', services: ['heart_rate']}],
    optionalServices: ['heart_rate']
  }
];

bluetooth_test(() => {
  let test_promises = Promise.resolve();
  test_specs.forEach(args => {
    test_promises = test_promises.then(
        () => assert_promise_rejects_with_message(
            requestDeviceWithTrustedClick(args), expected));
  });
  return test_promises;
}, test_desc);
