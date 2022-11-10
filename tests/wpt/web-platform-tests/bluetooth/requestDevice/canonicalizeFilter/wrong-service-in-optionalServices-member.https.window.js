// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Invalid optional service must reject the promise.';
const expected = new TypeError();
const test_specs = [
  {optionalServices: ['wrong_service'], filters: [{services: ['heart_rate']}]},
  {
    optionalServices: ['wrong_service'],
    filters: [{services: ['heart_rate'], name: 'Name'}]
  },
  {
    optionalServices: ['wrong_service'],
    filters: [{services: ['heart_rate'], namePrefix: 'Pre'}]
  },
  {
    optionalServices: ['wrong_service'],
    filters: [{services: ['heart_rate'], name: 'Name', namePrefix: 'Pre'}]
  },
  {optionalServices: ['wrong_service'], filters: [{name: 'Name'}]}, {
    optionalServices: ['wrong_service'],
    filters: [{name: 'Name', namePrefix: 'Pre'}]
  },
  {optionalServices: ['wrong_service'], filters: [{namePrefix: 'Pre'}]}
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
