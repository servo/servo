'use strict';

// These tests rely on the User Agent providing an implementation of
// platform sensor backends.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest
const loadChromiumResources = async () => {
  if (!('MojoInterfaceInterceptor' in self)) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  const resources = [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings.js',
    '/gen/mojo/public/mojom/base/string16.mojom.js',
    '/gen/services/device/public/mojom/sensor.mojom.js',
    '/gen/services/device/public/mojom/sensor_provider.mojom.js',
    '/resources/testdriver.js',
    '/resources/testdriver-vendor.js',
    '/resources/chromium/generic_sensor_mocks.js',
  ];

  await Promise.all(resources.map(path => {
    const script = document.createElement('script');
    script.src = path;
    script.async = false;
    const promise = new Promise((resolve, reject) => {
      script.onload = resolve;
      script.onerror = reject;
    });
    document.head.appendChild(script);
    return promise;
  }));
};

async function initialize_generic_sensor_tests() {
  if (typeof GenericSensorTest === 'undefined') {
    await loadChromiumResources();
  }
  assert_true(
    typeof GenericSensorTest !== 'undefined',
    'Mojo testing interface is not available.'
  );
  let sensorTest = new GenericSensorTest();
  await sensorTest.initialize();
  return sensorTest;
}

function sensor_test(func, name, properties) {
  promise_test(async (t) => {
    let sensorTest = await initialize_generic_sensor_tests();
    try {
      await func(t, sensorTest.getSensorProvider());
    } finally {
      await sensorTest.reset();
    };
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
