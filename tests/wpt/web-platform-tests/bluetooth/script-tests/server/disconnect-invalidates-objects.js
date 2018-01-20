'use strict';
const test_desc = 'Calls on services after we disconnect and connect again. '+
   'Should reject with InvalidStateError.';
let device, services;

bluetooth_test(() => getHealthThermometerDevice({
      filters: [{services: ['health_thermometer']}]
    })
    .then(_ => ({device} = _))
    .then(() => device.gatt.CALLS([
      getPrimaryService('health_thermometer')|
      getPrimaryServices()|
      getPrimaryServices('health_thermometer')[UUID]]))
    // Convert to array if necessary.
    .then(s => services = [].concat(s))
    .then(() => device.gatt.disconnect())
    .then(() => device.gatt.connect())
    .then(() => {
      let promises = Promise.resolve();
      for (let service of services) {
        let error = new DOMException(
          `Service with UUID ${service.uuid} is no longer valid. Remember ` +
          `to retrieve the service again after reconnecting.`,
          'InvalidStateError');
        promises = promises.then(() =>
            assert_promise_rejects_with_message(
                service.getCharacteristic('measurement_interval'),
                error));
        promises = promises.then(() =>
            assert_promise_rejects_with_message(
                service.getCharacteristics(),
                error));
        promises = promises.then(() =>
            assert_promise_rejects_with_message(
                service.getCharacteristics('measurement_interval'),
                error));
      }
      return promises;
    }), test_desc);
