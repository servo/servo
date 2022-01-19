// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'requestDevice with empty manufacturerData. ' +
    'Should reject with TypeError.';
const test_specs = [
  {filters: [{manufacturerData: []}]},
  {filters: [{manufacturerData: [], name: 'Name'}]},
  {filters: [{manufacturerData: [], services: ['heart_rate']}]},
  {filters: [{manufacturerData: [], name: 'Name', services: ['heart_rate']}]},
  {filters: [{manufacturerData: []}], optionalServices: ['heart_rate']}, {
    filters: [{manufacturerData: [], name: 'Name'}],
    optionalServices: ['heart_rate']
  },
  {
    filters: [{manufacturerData: [], services: ['heart_rate']}],
    optionalServices: ['heart_rate']
  },
  {
    filters: [{manufacturerData: [], name: 'Name', services: ['heart_rate']}],
    optionalServices: ['heart_rate']
  }
];

bluetooth_test((t) => {
  let test_promises = Promise.resolve();
  test_specs.forEach(args => {
    test_promises = test_promises.then(
        () => promise_rejects_js(
            t, TypeError, requestDeviceWithTrustedClick(args)));
  });
  return test_promises;
}, test_desc);