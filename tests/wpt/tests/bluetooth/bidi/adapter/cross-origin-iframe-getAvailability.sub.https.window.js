// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'getAvailability() resolves with false if called from a ' +
    'unique origin';
const cross_origin_src = 'https://{{domains[www]}}:{{ports[https][0]}}' +
    '/bluetooth/resources/health-thermometer-iframe.html'
let iframe = document.createElement('iframe');

bluetooth_bidi_test(async () => {
  await test_driver.bidi.bluetooth.simulate_adapter({state: "powered-on"});
  await new Promise(resolve => {
    iframe.src = cross_origin_src;
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });
  await new Promise(resolve => {
    callWithTrustedClick(
        () => iframe.contentWindow.postMessage({type: 'GetAvailability'}, '*'));

    window.onmessage = messageEvent => {
      assert_equals(
          messageEvent.data, false,
          'getAvailability resolves to false when called from a unique ' +
              'origin.');
      resolve();
    };
  });
}, test_desc);
