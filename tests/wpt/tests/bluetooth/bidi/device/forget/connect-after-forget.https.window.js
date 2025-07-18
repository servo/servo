// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long

bluetooth_bidi_test(async (t) => {
  const {device} = await getConnectedHealthThermometerDevice();
  await device.forget();

  await promise_rejects_dom(t, 'SecurityError', device.gatt.connect());
}, 'gatt.connect() rejects after forget().');
