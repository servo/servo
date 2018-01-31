'use strict';
const test_desc = 'Request for absent characteristics with UUID. ' +
    'Reject with NotFoundError.';

bluetooth_test(() => getEmptyHealthThermometerService()
    .then(({service}) => assert_promise_rejects_with_message(
        service.CALLS([
          getCharacteristic('battery_level')|
          getCharacteristics('battery_level')[UUID]
        ]),
        new DOMException(
            `No Characteristics matching UUID ${battery_level.uuid} found ` +
            `in Service with UUID ${health_thermometer.uuid}.`,
            'NotFoundError'))),
    test_desc);
