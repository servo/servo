'use strict';
const test_desc = 'Request for absent service. Must reject with ' +
    'NotFoundError even when the services have previously been discovered.';

bluetooth_test(() => getHealthThermometerDeviceWithServicesDiscovered({
      filters: [{services: ['health_thermometer']}],
      optionalServices: ['glucose']})
    .then(({device}) => assert_promise_rejects_with_message(
        device.gatt.CALLS([
          getPrimaryService('glucose')|
          getPrimaryServices('glucose')[UUID]
        ]),
        new DOMException(
            `No Services matching UUID ${glucose.uuid} found in Device.`,
            'NotFoundError'))),
    test_desc);
