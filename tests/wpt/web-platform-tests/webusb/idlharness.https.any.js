// META: timeout=long
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js

'use strict';

idl_test(
  ['webusb'],
  ['permissions', 'html', 'dom'],
  async idl_array => {
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({ Navigator: ['navigator'] });
    } else if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerNavigator: ['navigator'] });
    }

    idl_array.add_objects({
      USB: ['navigator.usb'],
      USBAlternateInterface: ['usbAlternateInterface'],
      USBConfiguration: ['usbConfiguration'],
      USBConnectionEvent: ['usbConnectionEvent'],
      USBDevice: ['usbDevice'],
      USBEndpoint: ['usbEndpoint'],
      USBInterface: ['usbInterface'],
      USBInTransferResult: ['new USBInTransferResult("ok")'],
      USBOutTransferResult: ['new USBOutTransferResult("ok")'],
      USBIsochronousInTransferResult: ['new USBIsochronousInTransferResult([])'],
      USBIsochronousOutTransferResult: ['new USBIsochronousOutTransferResult([])'],
      USBIsochronousInTransferPacket: ['new USBIsochronousInTransferPacket("ok")'],
      USBIsochronousOutTransferPacket: ['new USBIsochronousOutTransferPacket("ok")'],
    });

    return usb_test(async () => {
      // Ignored errors are surfaced in idlharness.js's test_object below.
      self.usbDevice = await getFakeDevice().device;
      self.usbConfiguration = usbDevice.configurations[0];
      self.usbInterface = usbConfiguration.interfaces[0];
      self.usbAlternateInterface = usbInterface.alternates[0];
      self.usbEndpoint = usbAlternateInterface.endpoints[0];
      self.usbConnectionEvent =
          new USBConnectionEvent('connect', { device: usbDevice });
    }, 'USB device setup');
  }
);
