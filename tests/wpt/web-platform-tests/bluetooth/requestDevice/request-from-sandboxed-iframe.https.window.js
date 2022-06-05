// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request device from a unique origin. ' +
    'Should reject with SecurityError.';
const expected = 'SecurityError: Failed to execute \'requestDevice\' on ' +
    '\'Bluetooth\': Access to the feature "bluetooth" is disallowed by ' +
    'permissions policy.';

let iframe = document.createElement('iframe');

bluetooth_test(
    () => getConnectedHealthThermometerDevice()
              // 1. Load the iframe.
              .then(() => new Promise(resolve => {
                      iframe.sandbox.add('allow-scripts');
                      iframe.src =
                          '/bluetooth/resources/health-thermometer-iframe.html';
                      document.body.appendChild(iframe);
                      iframe.addEventListener('load', resolve);
                    }))
              // 2. Request the device from the iframe.
              .then(() => new Promise(resolve => {
                      iframe.contentWindow.postMessage(
                          {type: 'RequestDevice'}, '*');

                      window.onmessage = messageEvent => {
                        assert_equals(messageEvent.data, expected);
                        resolve();
                      }
                    })),
    test_desc);
