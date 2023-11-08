'use strict';

const kGyroscopeSensorData = {
  sensorName: 'Gyroscope',
  permissionName: 'gyroscope',
  testDriverName: 'gyroscope',
  featurePolicyNames: ['gyroscope']
};

// Due to the gyroscope input values being rounded using a precision of
// 0.1 deg/sec, the expectedReadings and expectedRemappedReadings contain
// a significant number of decimal places.
const kGyroscopeReadings = {
  readings: [
    { x: 1, y: 2, z: 3 }
  ],
  expectedReadings: [
    { x: 1.00007366, y: 2.00014732, z: 3.00022098 }
  ],
  expectedRemappedReadings: [
    { x: -2.00014732, y: 1.00007366, z: 3.00022098 }
  ]
};
