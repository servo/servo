// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Two iframes in the same origin should be able to access ' +
    'each other\'s services';

const iframe1 = document.createElement('iframe');
const iframe2 = document.createElement('iframe');

function add_iframe(iframe) {
  let promise =
      new Promise(resolve => iframe.addEventListener('load', resolve));
  iframe.src = '/bluetooth/resources/health-thermometer-iframe.html';
  document.body.appendChild(iframe);
  return promise;
}

function send_message(iframe, command, arg, assert_func) {
  let promise = new Promise((resolve, reject) => {
    window.addEventListener('message', (messageEvent) => {
      try {
        assert_func(messageEvent.data);
      } catch (e) {
        reject(e);
      }
      resolve();
    }, {once: true});
  });
  if (command === 'RequestAndConnect') {
    arg = {filters: [{services: [arg]}]};
  }
  callWithTrustedClick(
      () => iframe.contentWindow.postMessage(
          {
            type: command,
            options: arg,
          },
          '*'));
  return promise;
}

bluetooth_test(async () => {
  await getHealthThermometerDevice();
  // 1. Add the first iframe.
  await add_iframe(iframe1);
  // 2. Connect with the first iframe, requesting the health
  // thermometer service.
  await send_message(
      iframe1, 'RequestAndConnect', 'health_thermometer',
      msg => assert_equals(msg, 'Connected'));
  // 3. Access the health thermometer service with the first iframe
  // (successfully).
  await send_message(
      iframe1, 'GetService', 'health_thermometer',
      msg => assert_equals(msg, 'ServiceReceived'));
  // 4. Access the generic access service with the first iframe
  // (unsuccessfully).
  await send_message(iframe1, 'GetService', 'generic_access', msg => {
    let split_msg = msg.split(': ');
    assert_equals(split_msg[0], 'FAIL');
    assert_equals(split_msg[1], 'SecurityError');
  });
  // 5. Add the second iframe.
  await add_iframe(iframe2);
  // 6. Connect with the second iframe, requesting the generic
  // access service.
  await send_message(
      iframe2, 'RequestAndConnect', 'generic_access',
      msg => assert_equals(msg, 'Connected'));
  // 7. Access the health thermometer service with the second iframe
  // (successfully).  Both iframes should have access to both
  // services at this point since they have the same origin.
  await send_message(
      iframe2, 'GetService', 'health_thermometer',
      msg => assert_equals(msg, 'ServiceReceived'));
  // 8. Access the generic access service with the second iframe
  // (unsuccessfully).
  await send_message(
      iframe2, 'GetService', 'generic_access',
      msg => assert_equals(msg, 'ServiceReceived'));
  // 9. Access the generic access service with the first iframe
  // (successfully).
  await send_message(
      iframe1, 'GetService', 'generic_access',
      msg => assert_equals(msg, 'ServiceReceived'));
}, test_desc);
