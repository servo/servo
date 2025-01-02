// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

usb_test(async () => {
  const {device} = await getFakeDevice();
  await device.open();

  for (const configuration of device.configurations) {
    await device.selectConfiguration(configuration.configurationValue);
    assert_equals(device.configuration, configuration);

    for (const interfaceObj of configuration.interfaces) {
      await device.claimInterface(interfaceObj.interfaceNumber);

      for (const alternate of interfaceObj.alternates) {
        await device.selectAlternateInterface(
            interfaceObj.interfaceNumber, alternate.alternateSetting);
        assert_equals(interfaceObj.alternate, alternate);
      }
      await device.releaseInterface(interfaceObj.interfaceNumber);
    }
  }
  await device.close();
}, '[SameObject] test for instances within USBDevice.');
