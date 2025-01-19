// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js

bluetooth_test(async () => {
  let iframe = document.createElement('iframe');
  let error;

  const {device, fakes} = await getHealthThermometerDeviceFromIframe(iframe);
  await fakes.fake_peripheral.setNextGATTDiscoveryResponse({
    code: HCI_SUCCESS,
  });
  let service = await device.gatt.getPrimaryService(health_thermometer.name);
  let characteristic =
      await service.getCharacteristic(measurement_interval.name);
  let descriptor = await characteristic.getDescriptor(user_description.name);

  iframe.remove();
  // Set iframe to null to ensure that the GC cleans up as much as possible.
  iframe = null;
  await garbageCollect();

  try {
    await descriptor.readValue();
  } catch (e) {
    // Cannot use promise_rejects_dom() because |e| is thrown from a different
    // global.
    error = e;
  }
  assert_not_equals(error, undefined);
  assert_equals(error.name, 'NetworkError');
}, 'readValue() rejects in a detached context');
