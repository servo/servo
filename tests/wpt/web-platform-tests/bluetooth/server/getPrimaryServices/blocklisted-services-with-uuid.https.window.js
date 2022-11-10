// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'Request for services. Does not return blocklisted service.';
const expected = new DOMException(
    'Origin is not allowed to access the service. Tip: Add the service ' +
        'UUID to \'optionalServices\' in requestDevice() options. ' +
        'https://goo.gl/HxfxSQ',
    'SecurityError');

bluetooth_test(async () => {
  let {device} = await getConnectedHIDDevice({
    filters: [{services: ['device_information']}],
    optionalServices: ['human_interface_device']
  });
  assert_promise_rejects_with_message(
      device.gatt.getPrimaryServices('human_interface_device'), expected)
}, test_desc);
