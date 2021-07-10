// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = `watchAdvertisements() enables 'advertisementreceived' ` +
    `events to be fired on the device object.`;

bluetooth_test(async (t) => {
  let {device} = await getDiscoveredHealthThermometerDevice();
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  await device.watchAdvertisements();
  assert_true(device.watchingAdvertisements);

  // This advertisement packet represents a different device and should not
  // cause an event to be fired on |device|.
  await fake_central.simulateAdvertisementReceived(heart_rate_ad_packet);

  let advertisementreceivedPromise = watcher.wait_for('advertisementreceived');
  await fake_central.simulateAdvertisementReceived(
      health_thermometer_ad_packet);
  let evt = await advertisementreceivedPromise;
  assert_equals(evt.device, device);
}, test_desc);
