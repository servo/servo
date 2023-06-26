'use strict';
const test_desc = 'Request for present service without permission to access ' +
    'any service. Reject with SecurityError.';
const expected = new DOMException(
    'Origin is not allowed to access any service. Tip: Add the service ' +
    'UUID to \'optionalServices\' in requestDevice() options. ' +
    'https://goo.gl/HxfxSQ',
     'SecurityError');

bluetooth_test(() => getConnectedHealthThermometerDevice({acceptAllDevices: true})
    .then(({device}) => assert_promise_rejects_with_message(
        device.gatt.CALLS([
          getPrimaryService('heart_rate')|
          getPrimaryServices()|
          getPrimaryServices('heart_rate')[UUID]]),
        expected)),
    test_desc);
