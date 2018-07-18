// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
// META: global=sharedworker
'use strict';

// Object instances used by the IDL test.
var usbDevice;
var usbConfiguration;
var usbInterface;
var usbAlternateInterface;
var usbEndpoint;
var usbConnectionEvent;

usb_test(async () => {
  // Ignored errors are surfaced in idlharness.js's test_object below.
  let device;
  try {
    device = (await getFakeDevice()).device;
  } catch (e) {}

  return idl_test(
    ['webusb'],
    ['html', 'dom'],
    idl_array => {
      // Untested IDL interfaces
      idl_array.add_untested_idls('dictionary PermissionDescriptor {};');
      idl_array.add_untested_idls('interface PermissionStatus {};');

      try {
        usbDevice = device;
        usbConfiguration = usbDevice.configurations[0];
        usbInterface = usbConfiguration.interfaces[0];
        usbAlternateInterface = usbInterface.alternates[0];
        usbEndpoint = usbAlternateInterface.endpoints[0];
        usbConnectionEvent =
            new USBConnectionEvent('connect', { device: usbDevice })
      } catch (e) {}

      if (self.GLOBAL.isWindow()) {
        idl_array.add_objects({ Navigator: ['navigator'] });
      } else if (self.GLOBAL.isWorker()) {
        idl_array.add_objects({ WorkerNavigator: ['navigator'] });
      }

      idl_array.add_objects({
        Navigator: ['navigator'],
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
      idl_array.prevent_multiple_testing('Navigator');
    },
    'WebUSB IDL test'
  )
});

done();
