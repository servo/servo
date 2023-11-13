'use strict';

const kGeolocationSensorData = {
  sensorName: 'GeolocationSensor',
  permissionName: 'geolocation',
  testDriverName: 'geolocation',
  featurePolicyNames: ['geolocation']
};

const kGeolocationReadings = {
  readings: [
      [1.12345, 2.12345, 3.12345, 0.95, 0.96, 4.12345, 5.123]
  ],
  expectedReadings: [
      [1.12345, 2.12345, 3.12345, 0.95, 0.96, 4.12345, 5.123]
  ]
};
