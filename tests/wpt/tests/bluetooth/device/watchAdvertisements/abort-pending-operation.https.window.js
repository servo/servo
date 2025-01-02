// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = 'AbortController stops a pending watchAdvertisements() ' +
    'operation.';

bluetooth_test(async (t) => {
  let {device} = await getDiscoveredHealthThermometerDevice();
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);
  let abortController = new AbortController();

  let watchAdvertisementsPromise =
      device.watchAdvertisements({signal: abortController.signal});
  abortController.abort();
  assert_false(device.watchingAdvertisements);
  await promise_rejects_dom(t, 'AbortError', watchAdvertisementsPromise);

  await fake_central.simulateAdvertisementReceived(
      health_thermometer_ad_packet);
}, test_desc);
