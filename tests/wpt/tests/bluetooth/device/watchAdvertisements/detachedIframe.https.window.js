// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js

bluetooth_test(async () => {
  let iframe = document.createElement('iframe');
  let error;

  const {device} = await getHealthThermometerDeviceFromIframe(iframe);

  iframe.remove();
  // Set iframe to null to ensure that the GC cleans up as much as possible.
  iframe = null;
  await garbageCollect();

  try {
    await device.watchAdvertisements();
  } catch (e) {
    // Cannot use promise_rejects_dom() because |e| is thrown from a different
    // global.
    error = e;
  }
  assert_not_equals(error, undefined);
  assert_equals(error.name, 'TypeError');
}, 'watchAdvertisements() rejects in a detached context');
