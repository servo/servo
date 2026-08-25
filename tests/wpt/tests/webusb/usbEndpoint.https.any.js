// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

usb_test(async () => {
  let { device } = await getFakeDevice();
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  let usbInterface = new USBInterface(
      configuration, configuration.interfaces[0].interfaceNumber);
  let alternateInterface = new USBAlternateInterface(
      usbInterface, usbInterface.alternates[1].alternateSetting);
  let inEndpoint = new USBEndpoint(
      alternateInterface, alternateInterface.endpoints[0].endpointNumber, 'in');
  let outEndpoint = new USBEndpoint(
      alternateInterface,
      alternateInterface.endpoints[1].endpointNumber,
      'out');
  assertDeviceInfoEquals(
      inEndpoint,
      fakeDeviceInit.configurations[1].interfaces[0].alternates[1]
          .endpoints[0]);
  assertDeviceInfoEquals(
      outEndpoint,
      fakeDeviceInit.configurations[1].interfaces[0].alternates[1]
          .endpoints[1]);
}, 'Can construct a USBEndpoint.');

usb_test(async () => {
  let { device } = await getFakeDevice();
  let configuration = new USBConfiguration(
      device, device.configurations[1].configurationValue);
  let usbInterface = new USBInterface(
      configuration, configuration.interfaces[0].interfaceNumber);
  let alternateInterface = new USBAlternateInterface(
      usbInterface, usbInterface.alternates[1].alternateSetting);
  try {
    let endpoint = new USBEndpoint(
        alternateInterface, alternateInterface.endpoints.length, 'in');
    assert_unreached('USBEndpoint should reject an invalid endpoint number');
  } catch (error) {
    assert_equals(error.name, 'RangeError');
  }
}, 'Constructing a USBEndpoint with an invalid endpoint number  throws a ' +
    'range error.');
