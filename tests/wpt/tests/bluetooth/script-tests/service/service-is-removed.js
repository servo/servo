'use strict';
const test_desc = 'Service is removed before FUNCTION_NAME call. ' +
    'Reject with InvalidStateError.';
const expected = new DOMException('GATT Service no longer exists.',
    'InvalidStateError');
let service, fake_service, fake_peripheral;

bluetooth_test(() => getHealthThermometerService()
    .then(_ => ({service, fake_service, fake_peripheral} = _))
    .then(() => fake_service.remove())
    .then(() => fake_peripheral.simulateGATTServicesChanged())
    .then(() => assert_promise_rejects_with_message(
        service.CALLS([
          getCharacteristic('measurement_interval')|
          getCharacteristics()|
          getCharacteristics('measurement_interval')[UUID]
        ]),
        expected,
        'Service got removed.')),
    test_desc);
