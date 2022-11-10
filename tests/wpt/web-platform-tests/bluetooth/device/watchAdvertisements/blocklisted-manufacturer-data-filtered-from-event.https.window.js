// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = `Blocked manufacturer data is filtered from the ` +
    `advertisement event.`;

const advertisement_packet_with_blocked_manufacturer_data = {
  deviceAddress: '07:07:07:07:07:07',
  rssi: -10,
  scanRecord: {
    name: 'LE Device',
    uuids: [uuid1234],
    manufacturerData: {
      [nonBlocklistedManufacturerId]: nonBlocklistedManufacturerData,
      [blocklistedManufacturerId]: blocklistedManufacturerData,
    },
  }
};

bluetooth_test(async (t) => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {
      address: '07:07:07:07:07:07',
      knownServiceUUIDs: [uuid1234],
    },
    requestDeviceOptions: {
      filters: [{services: [uuid1234]}],
      optionalManufacturerData: [nonBlocklistedManufacturerId, blocklistedManufacturerId]
    }
  });
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  await device.watchAdvertisements();
  assert_true(device.watchingAdvertisements);

  let advertisementreceivedPromise = watcher.wait_for('advertisementreceived');
  await fake_central.simulateAdvertisementReceived(
      advertisement_packet_with_blocked_manufacturer_data);
  let evt = await advertisementreceivedPromise;
  assert_equals(evt.device, device);

  // Check if block-listed manufacturer data is filtered out properly.
  assert_false(evt.manufacturerData.has(blocklistedManufacturerId));

  // Check if non blocked-listed manufacturer still exists.
  assert_data_maps_equal(
    evt.manufacturerData, /*expected_key=*/ nonBlocklistedManufacturerId, nonBlocklistedManufacturerData);
}, test_desc);
