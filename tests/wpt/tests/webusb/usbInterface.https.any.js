// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

usb_test(async () => {
  let { device } =  await getFakeDevice();
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  let usbInterface = new USBInterface(
      configuration, configuration.interfaces[0].interfaceNumber);
  assertDeviceInfoEquals(
      usbInterface, fakeDeviceInit.configurations[1].interfaces[0]);
}, 'Can construct a USBInterface.');

usb_test(async () => {
  let { device } =  await getFakeDevice();
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  try {
    let usbInterface = new USBInterface(
        configuration, configuration.interfaces.length);
    assert_unreached('USBInterface should reject an invalid interface number');
  } catch (error) {
    assert_equals(error.name, 'RangeError');
  }
}, 'Constructing a USBInterface with an invalid interface number ' +
    'throws a range error.');

usb_test(async () => {
  let { device } =  await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  let usbInterface = new USBInterface(
      configuration, configuration.interfaces[0].interfaceNumber);
  assert_equals(usbInterface.alternate.alternateSetting, 0);
}, 'The alternate attribute of USBInterface returns the one with ' +
   'bAlternateSetting 0 if the interface has not been claimed.');

usb_test(async () => {
  let { device } =  await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  let configuration = new USBConfiguration(
    device, device.configurations[1].configurationValue);
  let usbInterface = new USBInterface(
    configuration, configuration.interfaces[0].interfaceNumber);
  assert_equals(usbInterface.alternate.alternateSetting, 0);
  await device.selectAlternateInterface(0, 1);
  assert_equals(usbInterface.alternate.alternateSetting, 1);
}, 'The alternate attribute of USBInterface returns the active alternate ' +
   'interface.');
