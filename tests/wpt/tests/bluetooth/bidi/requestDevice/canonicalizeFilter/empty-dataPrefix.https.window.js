// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = 'dataPrefix when present must be non-empty';

bluetooth_bidi_test(async (t) => {
  await promise_rejects_js(
      t, TypeError, requestDeviceWithTrustedClick({
        filters: [{
          manufacturerData:
              [{companyIdentifier: 1, dataPrefix: new Uint8Array()}]
        }]
      }));
}, test_desc);
