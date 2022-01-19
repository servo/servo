// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'dataPrefix when present must be non-empty';

bluetooth_test(async (t) => {
  await promise_rejects_js(
      t, TypeError, requestDeviceWithTrustedClick({
        filters: [{
          manufacturerData:
              [{companyIdentifier: 1, dataPrefix: new Uint8Array()}]
        }]
      }));
}, test_desc);