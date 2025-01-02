// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js

'use strict';

let iframe = document.createElement('iframe');

bluetooth_test(async () => {
  await getConnectedHealthThermometerDevice();
  await new Promise(resolve => {
    iframe.src = '/bluetooth/resources/health-thermometer-iframe.html';
    iframe.sandbox.add('allow-scripts');
    iframe.allow = 'bluetooth';
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });
  await new Promise(resolve => {
    iframe.contentWindow.postMessage({type: 'RequestDevice'}, '*');

    window.addEventListener('message', (messageEvent) => {
      assert_false(/^FAIL: .*/.test(messageEvent.data));
      resolve();
    });
  });
}, 'Calls to Bluetooth APIs from a sandboxed iframe are valid.');