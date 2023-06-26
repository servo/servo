'use strict';
const test_desc = 'Calls to FUNCTION_NAME should return the same object.';

bluetooth_test(() => getHealthThermometerService()
    .then(({service}) => Promise.all([
      service.CALLS([
        getCharacteristic('measurement_interval')|
        getCharacteristics()|
        getCharacteristics('measurement_interval')[UUID]]),
      service.PREVIOUS_CALL]))
    .then(([characteristics_first_call, characteristics_second_call]) => {
      // Convert to arrays if necessary.
      characteristics_first_call = [].concat(characteristics_first_call);
      characteristics_second_call = [].concat(characteristics_second_call);

      let first_call_set = new Set(characteristics_first_call);
      assert_equals(characteristics_first_call.length, first_call_set.size);
      let second_call_set = new Set(characteristics_second_call);
      assert_equals(characteristics_second_call.length, second_call_set.size);

      characteristics_first_call.forEach(characteristic => {
        assert_true(second_call_set.has(characteristic));
      });
    }), test_desc);
