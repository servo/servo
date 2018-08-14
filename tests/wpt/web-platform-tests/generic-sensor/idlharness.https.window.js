// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/sensors/

'use strict';

function cast(i, t) {
  return Object.assign(Object.create(t.prototype), i);
}

idl_test(
  ['generic-sensor'],
  ['dom', 'html', 'WebIDL'],
  idl_array => {
    idl_array.add_objects({
      Sensor: ['cast(new Accelerometer(), Sensor)'],
      SensorErrorEvent: [
        'new SensorErrorEvent("error", { error: new DOMException });'
      ],
    });
  }
);
