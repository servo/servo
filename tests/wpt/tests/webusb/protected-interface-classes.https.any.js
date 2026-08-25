// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

async function runTestForInterfaceClass(t, interfaceClass) {
  await navigator.usb.test.initialize();

  const fakeDeviceTemplate = {
    usbVersionMajor: 2,
    usbVersionMinor: 0,
    usbVersionSubminor: 0,
    deviceClass: 7,
    deviceSubclass: 1,
    deviceProtocol: 2,
    vendorId: 0x18d1,
    productId: 0xf00d,
    deviceVersionMajor: 1,
    deviceVersionMinor: 2,
    deviceVersionSubminor: 3,
    manufacturerName: 'Google, Inc.',
    productName: 'Test Device',
    serialNumber: '4 (chosen randomly)',
    activeConfigurationValue: 0,
    configurations: [{
      configurationValue: 1,
      configurationName: 'Default configuration',
      interfaces: [{
        interfaceNumber: 0,
        alternates: [{
          alternateSetting: 0,
          interfaceClass: interfaceClass,
          interfaceSubclass: 0x01,
          interfaceProtocol: 0x01,
          interfaceName: 'Protected interface',
          endpoints: []
        }]
      }, {
        interfaceNumber: 1,
        alternates: [{
          alternateSetting: 0,
          interfaceClass: 0xff,
          interfaceSubclass: 0x01,
          interfaceProtocol: 0x01,
          interfaceName: 'Unprotected interface',
          endpoints: []
        }]
      }]
    }]
  };

  let fakeDevice;
  let device = await new Promise((resolve) => {
    navigator.usb.addEventListener('connect', (e) => {
      resolve(e.device);
    }, { once: true });
    fakeDevice = navigator.usb.test.addFakeDevice(fakeDeviceTemplate);
  });

  await device.open();
  await device.selectConfiguration(1);

  await promise_rejects_dom(t, 'SecurityError', device.claimInterface(0));
  await device.claimInterface(1);

  await device.close();
  fakeDevice.disconnect();
}

usb_test(
    (t) => runTestForInterfaceClass(t, 0x01),
    'Protected audio interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0x03),
    'Protected HID interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0x08),
    'Protected mass storage interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0x0B),
    'Protected smart card interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0x0E),
    'Protected video interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0x10),
    'Protected audio/video interface cannot be claimed');
usb_test(
    (t) => runTestForInterfaceClass(t, 0xE0),
    'Protected wireless controller interface cannot be claimed');
