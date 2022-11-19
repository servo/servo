// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Garbage collect then detach frame. We shouldn\'t crash.';
let iframe = document.createElement('iframe');

bluetooth_test(async () => {
  await setUpConnectableHealthThermometerDevice();
  // 1. Load the iframe.
  await new Promise(resolve => {
    iframe.src = '/bluetooth/resources/health-thermometer-iframe.html';
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });
  // 2. Connect device, run garbage collection, and detach iframe.
  await new Promise(resolve => {
    callWithTrustedClick(() => {
      iframe.contentWindow.postMessage(
          {
            type: 'RequestAndConnect',
            options: {filters: [{services: ['health_thermometer']}]}
          },
          '*');
    });
    window.onmessage = messageEvent => {
      assert_equals(messageEvent.data, 'Connected');
      garbageCollect().then(() => {
        iframe.remove();
        resolve();
      });
    }
  })
}, test_desc)
