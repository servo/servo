'use strict';
importScripts('/resources/testharness.js');
importScripts('/resources/WebIDLParser.js');
importScripts('/resources/idlharness.js');
importScripts('/webusb/resources/fake-devices.js');
importScripts('/webusb/resources/usb-helpers.js');

// Object instances used by the IDL test.
var usbDevice;
var usbConfiguration;
var usbInterface;
var usbAlternateInterface;
var usbEndpoint;
var usbConnectionEvent;

usb_test(async () => {
  let webUSBResponse = await fetch('/interfaces/webusb.idl');
  let domResponse = await fetch('/interfaces/dom.idl');
  let webusb_idl_text = await webUSBResponse.text();
  let dom_idl_text = await domResponse.text();
  let idl_array = new IdlArray();
  idl_array.add_idls(webusb_idl_text);

  // Untested IDL interfaces
  idl_array.add_untested_idls(dom_idl_text, { only: ['Event', 'EventTarget'] });
  idl_array.add_untested_idls('interface EventHandler {};');
  idl_array.add_untested_idls('dictionary EventInit {};');
  idl_array.add_untested_idls('interface Navigator {};');
  idl_array.add_untested_idls('interface WorkerNavigator {};');

  let {device} = await getFakeDevice();

  usbDevice = device;
  usbConfiguration = usbDevice.configurations[0];
  usbInterface = usbConfiguration.interfaces[0];
  usbAlternateInterface = usbInterface.alternates[0];
  usbEndpoint = usbAlternateInterface.endpoints[0];
  usbConnectionEvent =
      new USBConnectionEvent('connect', { device: usbDevice })

  idl_array.add_objects({
    WorkerNavigator: ['navigator'],
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

  idl_array.test();
}, 'WebUSB on Workers IDL test');

done();