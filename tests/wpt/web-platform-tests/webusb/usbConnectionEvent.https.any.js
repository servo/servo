// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js

'use strict';

usb_test(() => getFakeDevice()
    .then(({ device }) => {
      let evt = new USBConnectionEvent('connect', { device: device });
      assert_equals(evt.type, 'connect');
      assert_equals(evt.device, device);
    }),
    'Can construct a USBConnectionEvent with a device');

test(t => {
  assert_throws_js(TypeError,
      () => new USBConnectionEvent('connect', { device: null }));
  assert_throws_js(TypeError,
      () => new USBConnectionEvent('connect', {}));
}, 'Cannot construct a USBConnectionEvent without a device');

done();
