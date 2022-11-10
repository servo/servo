// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'requestDevice called with acceptAllDevices: true and with ' +
  'optionalServices. Should get access to services.';

bluetooth_test(
  async () => {
    await getTwoHealthThermometerServicesDevice()
    let device = await requestDeviceWithTrustedClick({
      acceptAllDevices: true,
      optionalServices: ['health_thermometer']
    });
    let gattServer = await device.gatt.connect();
    let services = await gattServer.getPrimaryServices();
    assert_equals(services.length, 2);
    services.forEach(service => {
      assert_equals(
        service.uuid,
        BluetoothUUID.getService('health_thermometer'));
    });
  },
  test_desc);
