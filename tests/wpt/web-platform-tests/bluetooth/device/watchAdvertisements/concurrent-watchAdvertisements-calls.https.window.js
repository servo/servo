// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'concurrent watchAdvertisements() calls results in the ' +
    `second call rejecting with 'InvalidStateError'`;

bluetooth_test(async (t) => {
  let {device} = await getDiscoveredHealthThermometerDevice();
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  // Start a watchAdvertisements() operation.
  let firstWatchAdvertisementsPromise = device.watchAdvertisements();

  // Start a second watchAdvertisements() operation. This operation should
  // reject with 'InvalidStateError'.
  await promise_rejects_dom(
      t, 'InvalidStateError', device.watchAdvertisements());

  // The first watchAdvertisements() operation should resolve successfully.
  await firstWatchAdvertisementsPromise;

  let advertisementreceivedPromise = watcher.wait_for('advertisementreceived');
  await fake_central.simulateAdvertisementReceived(
      health_thermometer_ad_packet);
  let evt = await advertisementreceivedPromise;
  assert_equals(evt.device, device);
}, test_desc);
