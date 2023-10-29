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
  readings: [{quaternion: [1, 0, 0, 0]}],
  expectedReadings: [{quaternion: [1, 0, 0, 0]}],
  expectedRemappedReadings: [{quaternion: [-0.70710678, 0.70710678, 0, 0]}]
};

const kRotationMatrix = [1,  0,  0, 0,
                         0, -1,  0, 0,
                         0,  0, -1, 0,
                         0,  0,  0, 1];
