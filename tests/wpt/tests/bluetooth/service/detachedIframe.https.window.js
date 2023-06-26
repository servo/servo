// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js

bluetooth_test(async () => {
  let iframe = document.createElement('iframe');
  const {device} = await getHealthThermometerDeviceFromIframe(iframe);
  let error;

  iframe.remove();
  // Set iframe to null to ensure that the GC cleans up as much as possible.
  iframe = null;
  await garbageCollect();

  try {
    await device.gatt.getPrimaryService(health_thermometer.name);
  } catch (e) {
    // Cannot use promise_rejects_dom() because |e| is thrown from a different
    // global.
    error = e;
  }
  assert_not_equals(error, undefined);
  assert_equals(error.name, 'NetworkError');
}, 'getPrimaryService() rejects in a detached context');
