'use strict';
const test_desc = 'Characteristic gets removed. Reject with InvalidStateError.';
const expected = new DOMException('GATT Characteristic no longer exists.',
                                  'InvalidStateError');
let fake_peripheral, characteristic, fake_characteristic;

bluetooth_test(() => getMeasurementIntervalCharacteristic()
    .then(_ => ({fake_peripheral, characteristic, fake_characteristic} = _))
    .then(() => characteristic.getDescriptor(user_description.name))
    .then(() => null, (e) => assert_unreached('Caught error unexpectedly.', e))
    .then(() => fake_characteristic.remove())
    .then(() => fake_peripheral.simulateGATTServicesChanged())
    .then(() => assert_promise_rejects_with_message(
      characteristic.CALLS([
        getDescriptor(user_description.name)|
        getDescriptors(user_description.name)[UUID]|
        getDescriptors()|
        readValue()|
        writeValue(new Uint8Array(1))|
        startNotifications()
      ]), expected)),
    test_desc);
