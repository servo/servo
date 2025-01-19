'use strict';
const test_desc = 'disconnect() called during a FUNCTION_NAME ' +
    'call that fails. Reject with NetworkError.';
const expected = new DOMException(
    'GATT Server is disconnected. Cannot retrieve services. (Re)connect ' +
    'first with `device.gatt.connect`.', 'NetworkError');
let device;

bluetooth_test(() => getEmptyHealthThermometerDevice()
    .then(_ => ({device} = _))
    .then(() => {
      let promise = assert_promise_rejects_with_message(
        device.gatt.CALLS([
          getPrimaryService('health_thermometer')|
          getPrimaryServices()|
          getPrimaryServices('health_thermometer')[UUID]
        ]),
        expected)
      device.gatt.disconnect();
      return promise;
    }),
    test_desc);
