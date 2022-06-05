// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request device from a unique origin. ' +
    'Should reject with SecurityError.';
const cross_origin_src = 'https://{{domains[www]}}:{{ports[https][0]}}' +
    '/bluetooth/resources/health-thermometer-iframe.html'
let iframe = document.createElement('iframe');

bluetooth_test(async (t) => {
  await setUpHealthThermometerDevice();

  // 1. Load the iframe.
  const iframeWatcher = new EventWatcher(t, iframe, ['load']);
  iframe.src = cross_origin_src;
  document.body.appendChild(iframe);
  await iframeWatcher.wait_for('load');

  // 2. Request the device from the iframe.
  const windowWatcher = new EventWatcher(t, window, ['message']);
  iframe.contentWindow.postMessage({type: 'RequestDevice'}, '*');
  const messageEvent = await windowWatcher.wait_for('message');
  assert_equals(
      messageEvent.data,
      'SecurityError: Failed to execute \'requestDevice\' on \'Bluetooth\': Access to the feature "bluetooth" is disallowed by permissions policy.');
}, test_desc);
