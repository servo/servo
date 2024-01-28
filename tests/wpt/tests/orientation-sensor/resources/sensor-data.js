'use strict';

const kAbsoluteOrientationSensorData = {
  sensorName: 'AbsoluteOrientationSensor',
  permissionName: 'accelerometer',
  testDriverName: 'absolute-orientation',
  featurePolicyNames: ['accelerometer', 'gyroscope', 'magnetometer']
};

const kRelativeOrientationSensorData = {
  sensorName: 'RelativeOrientationSensor',
  permissionName: 'accelerometer',
  testDriverName: 'relative-orientation',
  featurePolicyNames: ['accelerometer', 'gyroscope']
};

const kOrientationReadings = {
  // WebDriver input data must be given in Euler angles according to
  // https://w3c.github.io/deviceorientation/#parse-orientation-data-reading-algorithm
  // and converted to quaternions via
  // https://w3c.github.io/orientation-sensor/#create-a-quaternion-from-euler-angles.
  readings: [{alpha: 0, beta: -180, gamma: 0}],
  expectedReadings: [{quaternion: [-1, 0, 0, 0]}],
  expectedRemappedReadings: [{quaternion: [0.70710678, -0.70710678, 0, 0]}]
};

const kRotationMatrix = [1,  0,  0, 0,
                         0, -1,  0, 0,
                         0,  0, -1, 0,
                         0,  0,  0, 1];
