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
  const chromiumResources = [
    '/gen/mojo/public/mojom/base/string16.mojom.js',
    '/gen/services/device/public/mojom/sensor.mojom.js',
    '/gen/services/device/public/mojom/sensor_provider.mojom.js',
  ];
  await loadMojoResources(chromiumResources);
  await loadScript('/resources/testdriver.js');
  await loadScript('/resources/testdriver-vendor.js');
  await loadScript('/resources/chromium/generic_sensor_mocks.js');
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
