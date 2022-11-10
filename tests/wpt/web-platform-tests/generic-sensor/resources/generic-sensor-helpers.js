'use strict';

// These tests rely on the User Agent providing an implementation of
// platform sensor backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest
async function loadChromiumResources() {
  await loadScript('/resources/testdriver.js');
  await loadScript('/resources/testdriver-vendor.js');
  await import('/resources/chromium/generic_sensor_mocks.js');
}

async function initialize_generic_sensor_tests() {
  if (typeof GenericSensorTest === 'undefined') {
    const script = document.createElement('script');
    script.src = '/resources/test-only-api.js';
    script.async = false;
    const p = new Promise((resolve, reject) => {
      script.onload = () => { resolve(); };
      script.onerror = e => { reject(e); };
    })
    document.head.appendChild(script);
    await p;

    if (isChromiumBased) {
      await loadChromiumResources();
    }
  }

  let sensorTest = new GenericSensorTest();
  await sensorTest.initialize();
  return sensorTest;
}

function sensor_test(func, name, properties) {
  promise_test(async (t) => {
    t.add_cleanup(() => {
      if (sensorTest)
        return sensorTest.reset();
    });

    let sensorTest = await initialize_generic_sensor_tests();
    return func(t, sensorTest.getSensorProvider());
  }, name, properties);
}

function verifySensorReading(pattern, values, timestamp, isNull) {
  // If |val| cannot be converted to a float, we return the original value.
  // This can happen when a value in |pattern| is not a number.
  function round(val) {
    const res = Number.parseFloat(val).toPrecision(6);
    return res === "NaN" ? val : res;
  }

  if (isNull) {
    return (values === null || values.every(r => r === null)) &&
           timestamp === null;
  }

  return values.every((r, i) => round(r) === round(pattern[i])) &&
         timestamp !== null;
}

function verifyXyzSensorReading(pattern, {x, y, z, timestamp}, isNull) {
  return verifySensorReading(pattern, [x, y, z], timestamp, isNull);
}

function verifyQuatSensorReading(pattern, {quaternion, timestamp}, isNull) {
  return verifySensorReading(pattern, quaternion, timestamp, isNull);
}

function verifyAlsSensorReading(pattern, {illuminance, timestamp}, isNull) {
  return verifySensorReading(pattern, [illuminance], timestamp, isNull);
}

function verifyGeoSensorReading(pattern, {latitude, longitude, altitude,
  accuracy, altitudeAccuracy, heading, speed, timestamp}, isNull) {
  return verifySensorReading(pattern, [latitude, longitude, altitude,
    accuracy, altitudeAccuracy, heading, speed], timestamp, isNull);
}

function verifyProximitySensorReading(pattern, {distance, max, near, timestamp}, isNull) {
  return verifySensorReading(pattern, [distance, max, near], timestamp, isNull);
}

// Assert that two Sensor objects have the same properties and values.
//
// Verifies that ``actual`` and ``expected`` have the same sensor properties
// and, if so, that their values are the same.
//
// @param {Sensor} actual - Test value.
// @param {Sensor} expected - Expected value.
function assert_sensor_equals(actual, expected) {
  assert_true(
      actual instanceof Sensor,
      'assert_sensor_equals: actual must be a Sensor');
  assert_true(
      expected instanceof Sensor,
      'assert_sensor_equals: expected must be a Sensor');

  // These properties vary per sensor type.
  const CUSTOM_PROPERTIES = [
    ['illuminance'], ['quaternion'], ['x', 'y', 'z'],
    [
      'latitude', 'longitude', 'altitude', 'accuracy', 'altitudeAccuracy',
      'heading', 'speed'
    ]
  ];

  // These properties are present on all objects derived from Sensor.
  const GENERAL_PROPERTIES = ['timestamp'];

  for (let customProperties of CUSTOM_PROPERTIES) {
    if (customProperties.every(p => p in actual) &&
        customProperties.every(p => p in expected)) {
      customProperties.forEach(p => {
        if (customProperties == 'quaternion') {
          assert_array_equals(
              actual[p], expected[p],
              `assert_sensor_equals: property '${p}' does not match`);
        } else {
          assert_equals(
              actual[p], expected[p],
              `assert_sensor_equals: property '${p}' does not match`);
        }
      });
      GENERAL_PROPERTIES.forEach(p => {
        assert_equals(
            actual[p], expected[p],
            `assert_sensor_equals: property '${p}' does not match`);
      });
      return;
    }
  }

  assert_true(false, 'assert_sensor_equals: sensors have different attributes');
}
