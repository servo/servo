'use strict';

// If two doubles differ by less than this amount, we can consider them
// to be effectively equal.
const kEpsilon = 1e-8;

class RingBuffer {
  constructor(data) {
    if (!Array.isArray(data)) {
      throw new TypeError('`data` must be an array.');
    }

    this.bufferPosition_ = 0;
    this.data_ = Array.from(data);
  }

  get data() {
    return Array.from(this.data_);
  }

  next() {
    const value = this.data_[this.bufferPosition_];
    this.bufferPosition_ = (this.bufferPosition_ + 1) % this.data_.length;
    return {done: false, value: value};
  }

  value() {
    return this.data_[this.bufferPosition_];
  }

  [Symbol.iterator]() {
    return this;
  }

  reset() {
    this.bufferPosition_ = 0;
  }
};

// Calls test_driver.update_virtual_sensor() until it results in a "reading"
// event. It waits |timeoutInMs| before considering that an event has not been
// delivered.
async function update_virtual_sensor_until_reading(
    t, readings, readingPromise, testDriverName, timeoutInMs) {
  while (true) {
    await test_driver.update_virtual_sensor(
        testDriverName, readings.next().value);
    const value = await Promise.race([
      new Promise(
          resolve => {t.step_timeout(() => resolve('TIMEOUT'), timeoutInMs)}),
      readingPromise,
    ]);
    if (value !== 'TIMEOUT') {
      break;
    }
  }
}

// This could be turned into a t.step_wait() call once
// https://github.com/web-platform-tests/wpt/pull/34289 is merged.
async function wait_for_virtual_sensor_state(testDriverName, predicate) {
  const result =
      await test_driver.get_virtual_sensor_information(testDriverName);
  if (!predicate(result)) {
    await wait_for_virtual_sensor_state(testDriverName, predicate);
  }
}

function validate_sensor_data(sensorData) {
  if (!('sensorName' in sensorData)) {
    throw new TypeError('sensorData.sensorName is missing');
  }
  if (!('permissionName' in sensorData)) {
    throw new TypeError('sensorData.permissionName is missing');
  }
  if (!('testDriverName' in sensorData)) {
    throw new TypeError('sensorData.testDriverName is missing');
  }
  if (sensorData.featurePolicyNames !== undefined &&
      !Array.isArray(sensorData.featurePolicyNames)) {
    throw new TypeError('sensorData.featurePolicyNames must be an array');
  }
}

function validate_reading_data(readingData) {
  if (!Array.isArray(readingData.readings)) {
    throw new TypeError('readingData.readings must be an array.');
  }
  if (!Array.isArray(readingData.expectedReadings)) {
    throw new TypeError('readingData.expectedReadings must be an array.');
  }
  if (readingData.readings.length < readingData.expectedReadings.length) {
    throw new TypeError(
        'readingData.readings\' length must be bigger than ' +
        'or equal to readingData.expectedReadings\' length.');
  }
  if (readingData.expectedRemappedReadings &&
      !Array.isArray(readingData.expectedRemappedReadings)) {
    throw new TypeError(
        'readingData.expectedRemappedReadings must be an ' +
        'array.');
  }
  if (readingData.expectedRemappedReadings &&
      readingData.expectedReadings.length !=
          readingData.expectedRemappedReadings.length) {
    throw new TypeError(
        'readingData.expectedReadings and ' +
        'readingData.expectedRemappedReadings must have the same ' +
        'length.');
  }
}

function get_sensor_reading_properties(sensor) {
  const className = sensor[Symbol.toStringTag];
  if ([
        'Accelerometer', 'GravitySensor', 'Gyroscope',
        'LinearAccelerationSensor', 'Magnetometer', 'ProximitySensor'
      ].includes(className)) {
    return ['x', 'y', 'z'];
  } else if (className == 'AmbientLightSensor') {
    return ['illuminance'];
  } else if ([
               'AbsoluteOrientationSensor', 'RelativeOrientationSensor'
             ].includes(className)) {
    return ['quaternion'];
  } else {
    throw new TypeError(`Unexpected sensor '${className}'`);
  }
}

// Checks that `sensor` and `expectedSensorLike` have the same properties
// (except for timestamp) and they have the same values.
//
// Options allows configuring some aspects of the comparison:
// - ignoreTimestamps (boolean): If true, `sensor` and `expectedSensorLike`'s
//   "timestamp" attribute will not be compared. If `expectedSensorLike` does
//   not have a "timestamp" attribute, the values will not be compared either.
//   This is particularly useful when comparing sensor objects from different
//   origins (and consequently different time origins).
function assert_sensor_reading_equals(
    sensor, expectedSensorLike, options = {}) {
  for (const prop of get_sensor_reading_properties(sensor)) {
    assert_true(
        prop in expectedSensorLike,
        `expectedSensorLike must have a property called '${prop}'`);
    if (Array.isArray(sensor[prop]))
      assert_array_approx_equals(
          sensor[prop], expectedSensorLike[prop], kEpsilon);
    else
      assert_approx_equals(sensor[prop], expectedSensorLike[prop], kEpsilon);
  }
  assert_not_equals(sensor.timestamp, null);

  if ('timestamp' in expectedSensorLike && !options.ignoreTimestamps) {
    assert_equals(
        sensor.timestamp, expectedSensorLike.timestamp,
        'Sensor timestamps must be equal');
  }
}

function assert_sensor_reading_is_null(sensor) {
  for (const prop of get_sensor_reading_properties(sensor)) {
    assert_equals(sensor[prop], null);
  }
  assert_equals(sensor.timestamp, null);
}

function serialize_sensor_data(sensor) {
  const sensorData = {};
  for (const property of get_sensor_reading_properties(sensor)) {
    sensorData[property] = sensor[property];
  }
  sensorData['timestamp'] = sensor.timestamp;

  // Note that this is not serialized by postMessage().
  sensorData[Symbol.toStringTag] = sensor[Symbol.toStringTag];

  return sensorData;
}
