// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'subsequent watchAdvertisements() calls result in the ' +
    'second call resolving successfully.';

bluetooth_test(async (t) => {
  let {device} = await getDiscoveredHealthThermometerDevice();
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  // Start a watchAdvertisements() operation.
  await device.watchAdvertisements();

  // Start a second watchAdvertisements() operation after the first one
  // resolves. This operation should resolve successfully.
  await device.watchAdvertisements();

  let advertisementreceivedPromise = watcher.wait_for('advertisementreceived');
  await fake_central.simulateAdvertisementReceived(
      health_thermometer_ad_packet);
  let evt = await advertisementreceivedPromise;
  assert_equals(evt.device, device);
}, test_desc);
