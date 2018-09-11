// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

usb_test(async () => {
  let { device } = await getFakeDevice();
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  assertDeviceInfoEquals(configuration, fakeDeviceInit.configurations[1]);
}, 'Can construct a USBConfiguration.');

usb_test(async () => {
  let { device } = await getFakeDevice();
  try {
    let configuration =
        new USBConfiguration(device, device.configurations.length + 1);
    assert_unreached(
        'USBConfiguration should reject an invalid configuration value');
  } catch (error) {
    assert_equals(error.name, 'RangeError');
  }
}, 'Constructing a USBConfiguration with an invalid configuration value ' +
    'throws a range error.');
