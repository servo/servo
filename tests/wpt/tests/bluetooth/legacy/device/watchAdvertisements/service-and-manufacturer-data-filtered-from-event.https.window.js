// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
'use strict';
const test_desc = `Service and Manufacturer that were not granted with ` +
    `requestDevice() are filtered from the advertisement event.`;

bluetooth_test(async (t) => {
  let {device} = await setUpPreconnectedFakeDevice({
    fakeDeviceOptions: {
      address: '07:07:07:07:07:07',
      knownServiceUUIDs: [uuid1234, uuid5678, uuidABCD],
    },
    requestDeviceOptions: {
      filters: [{services: [uuid1234]}],
      optionalServices: [uuid5678],
      optionalManufacturerData: [0x0001]
    }
  });
  const watcher = new EventWatcher(t, device, ['advertisementreceived']);

  await device.watchAdvertisements();
  assert_true(device.watchingAdvertisements);

  let advertisementreceivedPromise = watcher.wait_for('advertisementreceived');
  await fake_central.simulateAdvertisementReceived(
      service_and_manufacturer_data_ad_packet);
  let evt = await advertisementreceivedPromise;
  assert_equals(evt.device, device);

  // Check that service data is filtered out properly.
  assert_data_maps_equal(evt.serviceData, uuid1234, uuid1234Data);
  assert_data_maps_equal(evt.serviceData, uuid5678, uuid5678Data);
  assert_false(evt.serviceData.has(uuidABCD));

  // Check that manufacturer data is filtered out properly.
  assert_data_maps_equal(
      evt.manufacturerData, /*expected_key=*/ 0x0001, manufacturer1Data);
  assert_false(evt.manufacturerData.has(0x0002));
}, test_desc);
