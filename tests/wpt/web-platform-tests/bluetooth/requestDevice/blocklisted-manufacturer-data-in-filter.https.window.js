// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Reject with SecurityError if requesting a blocklisted ' +
    'manufacturer data.';

const expected = new DOMException(
    'requestDevice() called with a filter containing a blocklisted UUID ' +
        'or manufacturer data. https://goo.gl/4NeimX',
    'SecurityError');

bluetooth_test(async () => {
  await assert_promise_rejects_with_message(
      setUpPreconnectedFakeDevice({
        fakeDeviceOptions: {knownServiceUUIDs: ['heart_rate']},
        requestDeviceOptions: {
          filters: [{
            services: ['heart_rate'],
            manufacturerData: [{
              companyIdentifier: blocklistedManufacturerId,
              dataPrefix: blocklistedManufacturerData,
            }],
          }]
        }
      }),
      expected, 'Requesting blocklisted service rejects.');
}, test_desc);
