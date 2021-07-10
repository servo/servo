'use strict';
const test_desc = 'FUNCTION_NAME called before connecting. Reject with ' +
    'NetworkError.';
const expected = new DOMException(
    'GATT Server is disconnected. Cannot retrieve services. (Re)connect ' +
    'first with `device.gatt.connect`.',
    'NetworkError');

bluetooth_test(() => getDiscoveredHealthThermometerDevice({
      filters: [{services: ['health_thermometer']}],
      optionalServices: ['generic_access']
    })
    .then(({device}) => assert_promise_rejects_with_message(
        device.gatt.CALLS([
          getPrimaryService('health_thermometer')|
          getPrimaryServices()|
          getPrimaryServices('health_thermometer')[UUID]
        ]),
        expected)),
    test_desc);
