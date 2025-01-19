// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'companyIdentifier must be in the [0, 65535] range';

bluetooth_test(async (t) => {
  await promise_rejects_js(
      t, TypeError,
      requestDeviceWithTrustedClick(
          {filters: [{manufacturerData: [{companyIdentifier: -1}]}]}));
  await promise_rejects_js(
      t, TypeError,
      requestDeviceWithTrustedClick(
          {filters: [{manufacturerData: [{companyIdentifier: 65536}]}]}));
}, test_desc);