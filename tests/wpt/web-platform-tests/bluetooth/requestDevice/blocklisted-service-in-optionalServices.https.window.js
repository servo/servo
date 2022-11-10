// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Blocklisted UUID in optionalServices is removed and ' +
    'access not granted.';
const expected = new DOMException(
    'Origin is not allowed to access the ' +
        'service. Tip: Add the service UUID to \'optionalServices\' in ' +
        'requestDevice() options. https://goo.gl/HxfxSQ',
    'SecurityError');

bluetooth_test(async () => {
  let {device, fake_peripheral} = await getDiscoveredHealthThermometerDevice({
    filters: [{services: ['health_thermometer']}],
    optionalServices: ['human_interface_device']
  });
  await fake_peripheral.setNextGATTConnectionResponse({code: HCI_SUCCESS});
  await device.gatt.connect();
  Promise.all([
    assert_promise_rejects_with_message(
        device.gatt.getPrimaryService('human_interface_device'), expected,
        'Blocklisted service not accessible.'),
    assert_promise_rejects_with_message(
        device.gatt.getPrimaryServices('human_interface_device'), expected,
        'Blocklisted services not accessible.')
  ])
}, test_desc);
