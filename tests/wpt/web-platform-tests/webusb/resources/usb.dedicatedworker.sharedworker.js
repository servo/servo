'use strict';
importScripts('/resources/testharness.js');
importScripts('/webusb/resources/fake-devices.js');
importScripts('/webusb/resources/usb-helpers.js');

let usbDevice, devicesFirstTime, fakeDevice, removedDevice;

test(() => {
  assert_true(navigator.usb instanceof USB,
      'navigator.usb should be defined as a USB object');
  assert_true(navigator.usb.getDevices instanceof Function,
      'navigator.usb.getDevices should be defined as a function');
  assert_equals(typeof navigator.usb.requestDevice, 'undefined',
      'navigator.usb.requestDevice should not be defined');
}, 'Web workers should have the WebUSB API exposed as defined in the spec.');

usb_test(() => getFakeDevice()
    .then(_ => usbDevice = _.device)
    .then(() => navigator.usb.getDevices())
    .then(devices => {
      assert_equals(devices.length, 1);
      assert_equals(usbDevice, devices[0]);
      assertDeviceInfoEquals(devices[0], fakeDeviceInit);
    }), 'getDevices returns devices that are connected');

usb_test(() => getFakeDevice()
    .then(() => navigator.usb.getDevices())
    .then(_ => devicesFirstTime = _)
    .then(() => assert_equals(devicesFirstTime.length, 1))
    .then(() => navigator.usb.getDevices())
    .then(devicesSecondTime => assert_array_equals(devicesSecondTime,
        devicesFirstTime)),
    'getDevices returns the same objects for each USB device');

usb_test(() => getFakeDevice()
    .then(_ => usbDevice = _.device)
    .then(() => assertDeviceInfoEquals(usbDevice, fakeDeviceInit))
    .then(() => usbDevice.open())
    .then(() => usbDevice.close()),
    'onconnect event is trigged by adding a device');

usb_test(() => getFakeDevice()
    .then(_ => {
      usbDevice = _.device;
      fakeDevice = _.fakeDevice;
    })
    .then(() => waitForDisconnect(fakeDevice))
    .then(_ => removedDevice = _)
    .then(() => {
      assertDeviceInfoEquals(removedDevice, fakeDeviceInit);
      assert_equals(removedDevice, usbDevice);
    })
    .then(() => removedDevice.open())
    .then(() =>
        assert_unreachable('should not be able to open a disconnected device'),
        error => assert_equals(error.code, DOMException.NOT_FOUND_ERR)),
    'ondisconnect event is triggered by removing a device');

done();