'use strict';

const kSensorData = {
  sensorName: 'AmbientLightSensor',
  permissionName: 'ambient-light-sensor',
  testDriverName: 'ambient-light',
  featurePolicyNames: ['ambient-light-sensor']
};

const kReadings = {
  readings: [
    // Readings are selected so that illuminance significance check causes
    // the following to happen:
    // 1. First two values test situation when two values would be rounded
    //    to same value. As the second value would be rounded to same value
    //    as first it won't trigger reading event.
    // 2. New value is set to 24. And test checks it is correctly rounded to
    //    0.
    // 3. New reading is attempted to set to 35.
    // 4. Value is read from sensor and compared new reading. But as new
    //    reading was not significantly different compared to initial, for
    //    privacy reasons, service returns the initial value.
    // 5. New value is set to 49. And test checks it is correctly rounded to
    //    50. New value is allowed as it is significantly different compared
    //    to old value (24).
    // 6. New reading is attempted to set to 35.
    // 7. Value is read from sensor and compared new reading. But as new
    //    reading was not significantly different compared to initial, for
    //    privacy reasons, service returns the initial value.
    // 8. New value is set to 23. And test checks it is correctly rounded to
    //    0. New value is allowed as it is significantly different compared
    //    to old value (49).
    //
    // Note: Readings and expectedReadings wraps around correctly as next
    // value would be 150 (output from 127).
    { illuminance: 127 }, { illuminance: 165 }, { illuminance: 24 }, {
      illuminance:
        35
    }, { illuminance: 49 }, { illuminance: 35 }, { illuminance: 23 }
  ],
  expectedReadings: [
    { illuminance: 150 }, // output from 127
    { illuminance: 0 },   // output from 24
    { illuminance: 50 },  // output from 49
    { illuminance: 0 }    // output from 23
  ]
};
