// TODO(https://crbug.com/672127) Use this test case to test the rest of
// characteristic functions.
'use strict';
const test_desc = 'Service is removed. Reject with InvalidStateError.';
const expected = new DOMException('GATT Service no longer exists.',
    'InvalidStateError');
let characteristic, fake_peripheral, fake_service;

bluetooth_test(() => getMeasurementIntervalCharacteristic()
    .then(_ => ({characteristic, fake_peripheral, fake_service} = _))
    .then(() => fake_service.remove())
    .then(() => fake_peripheral.simulateGATTServicesChanged())
    .then(() => assert_promise_rejects_with_message(
        characteristic.CALLS([
          getDescriptor(user_description.name)|
          getDescriptors(user_description.uuid)[UUID]|
          getDescriptors(user_description.name)]),
        expected,
        'Service got removed.')),
    test_desc);
