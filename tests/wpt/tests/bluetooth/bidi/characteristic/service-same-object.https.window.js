// META: script=/resources/testdriver.js?feature=bidi
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-test.js
// META: script=/bluetooth/resources/bluetooth-fake-devices.js
// META: timeout=long
'use strict';
const test_desc = '[SameObject] test for BluetoothRemoteGATTCharacteristic ' +
    'service.';

bluetooth_bidi_test(async () => {
  const {characteristic} = await getMeasurementIntervalCharacteristic();
  assert_equals(characteristic.service, characteristic.service);
}, test_desc);
