// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'AbortController on subsequent watchAdvertisements() call ' +
    'cancels the watch advertisements operation.';

bluetooth_test(async (t) => {
  let {device} = await getDiscoveredHealthThermometerDevice();
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  // Start a watchAdvertisements() operation.
  await device.watchAdvertisements();
  assert_true(device.watchingAdvertisements);

  // Start a second watchAdvertisements() operation after the first one
  // resolves. This operation should resolve successfully.
  let abortController = new AbortController();
  await device.watchAdvertisements({signal: abortController.signal});
  abortController.abort();
  assert_false(device.watchingAdvertisements);

  // This advertisement packet should not cause the event to be fired.
  await fake_central.simulateAdvertisementReceived(heart_rate_ad_packet);
}, test_desc);
