'use strict';

const kAccelerometerSensorData = {
  sensorName: 'Accelerometer',
  permissionName: 'accelerometer',
  testDriverName: 'accelerometer',
  featurePolicyNames: ['accelerometer']
};

const kGravitySensorData = {
  sensorName: 'GravitySensor',
  permissionName: 'accelerometer',
  testDriverName: 'gravity',
  featurePolicyNames: ['accelerometer']
};

const kLinearAccelerationSensorData = {
  sensorName: 'LinearAccelerationSensor',
  permissionName: 'accelerometer',
  testDriverName: 'linear-acceleration',
  featurePolicyNames: ['accelerometer']
};

const kAccelerometerReadings = {
  readings: [
    { x: 1.12345, y: 2.12345, z: 3.12345 }
  ],
  expectedReadings: [
    { x: 1.1, y: 2.1, z: 3.1 }
  ],
  expectedRemappedReadings: [
    { x: -2.1, y: 1.1, z: 3.1 }
  ]
};
