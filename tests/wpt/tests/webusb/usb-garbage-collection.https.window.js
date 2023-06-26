// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
// META: script=/common/gc.js
'use strict';

usb_test(async () => {
  {
    let {device} = await getFakeDevice();
    await device.open();
    await device.selectConfiguration(2);
    await device.claimInterface(0);
  }
  await garbageCollect();
}, 'Run garbage collection when the device reference is out of scope');
