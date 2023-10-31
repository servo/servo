'use strict';

const kMagnetometerSensorData = {
  sensorName: 'Magnetometer',
  permissionName: 'magnetometer',
  testDriverName: 'magnetometer',
  featurePolicyNames: ['magnetometer']
};

const kUncalibratedMagnetometerSensorData = {
  sensorName: 'UncalibratedMagnetometer',
  permissionName: 'magnetometer',
  testDriverName: 'magnetometer',
  featurePolicyNames: ['magnetometer']
};

const kMagnetometerReadings = {
  readings: [{x: -19.2, y: 12.1, z: -44.3}],
  expectedReadings: [{x: -19.2, y: 12.1, z: -44.3}],
  expectedRemappedReadings: [{x: -12.1, y: -19.2, z: -44.3}]
};
