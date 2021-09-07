// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc =
    'Manufacturer data mask size must be equal to dataPrefix size.';

bluetooth_test(async (t) => {
  const companyIdentifier = 0x0001;
  const dataPrefix = new Uint8Array([0x01, 0x02, 0x03, 0x04]);
  const mask = new Uint8Array([0xff]);

  await promise_rejects_js(
      t, TypeError,
      requestDeviceWithTrustedClick(
          {filters: [{manufacturerData: [{companyIdentifier, mask}]}]}));
  await promise_rejects_js(
      t, TypeError, requestDeviceWithTrustedClick({
        filters: [{manufacturerData: [{companyIdentifier, dataPrefix, mask}]}]
      }));
}, test_desc);
