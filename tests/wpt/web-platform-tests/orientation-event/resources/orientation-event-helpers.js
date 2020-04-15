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

const MOTION_ROTATION_EPSILON = 1e-8;

function generateMotionData(accelerationX, accelerationY, accelerationZ,
                            accelerationIncludingGravityX,
                            accelerationIncludingGravityY,
                            accelerationIncludingGravityZ,
                            rotationRateAlpha, rotationRateBeta, rotationRateGamma,
                            interval = 16) {
  const motionData = {accelerationX: accelerationX,
                    accelerationY: accelerationY,
                    accelerationZ: accelerationZ,
                    accelerationIncludingGravityX: accelerationIncludingGravityX,
                    accelerationIncludingGravityY: accelerationIncludingGravityY,
                    accelerationIncludingGravityZ: accelerationIncludingGravityZ,
                    rotationRateAlpha: rotationRateAlpha,
                    rotationRateBeta: rotationRateBeta,
                    rotationRateGamma: rotationRateGamma,
                    interval: interval};
  return motionData;
}

function generateOrientationData(alpha, beta, gamma, absolute) {
  const orientationData = {alpha: alpha,
                         beta: beta,
                         gamma: gamma,
                         absolute: absolute};
  return orientationData;
}

async function setMockSensorDataForType(sensorProvider, sensorType, mockDataArray) {
  const createdSensor = await sensorProvider.getCreatedSensor(sensorType);
  return createdSensor.setSensorReading([mockDataArray]);
}

// Device[Orientation|Motion]EventPump treat NaN as a missing value.
let nullToNan = x => (x === null ? NaN : x);

function setMockMotionData(sensorProvider, motionData) {
  const degToRad = Math.PI / 180;
  return Promise.all([
      setMockSensorDataForType(sensorProvider, "Accelerometer", [
          nullToNan(motionData.accelerationIncludingGravityX),
          nullToNan(motionData.accelerationIncludingGravityY),
          nullToNan(motionData.accelerationIncludingGravityZ),
      ]),
      setMockSensorDataForType(sensorProvider, "LinearAccelerationSensor", [
          nullToNan(motionData.accelerationX),
          nullToNan(motionData.accelerationY),
          nullToNan(motionData.accelerationZ),
      ]),
      setMockSensorDataForType(sensorProvider, "Gyroscope", [
          nullToNan(motionData.rotationRateAlpha) * degToRad,
          nullToNan(motionData.rotationRateBeta) * degToRad,
          nullToNan(motionData.rotationRateGamma) * degToRad,
      ]),
  ]);
}

function setMockOrientationData(sensorProvider, orientationData) {
  let sensorType = orientationData.absolute
      ? "AbsoluteOrientationEulerAngles" : "RelativeOrientationEulerAngles";
  return setMockSensorDataForType(sensorProvider, sensorType, [
      nullToNan(orientationData.beta),
      nullToNan(orientationData.gamma),
      nullToNan(orientationData.alpha),
  ]);
}

function checkMotion(event, expectedMotionData) {
  assert_equals(event.acceleration.x, expectedMotionData.accelerationX, "acceleration.x");
  assert_equals(event.acceleration.y, expectedMotionData.accelerationY, "acceleration.y");
  assert_equals(event.acceleration.z, expectedMotionData.accelerationZ, "acceleration.z");

  assert_equals(event.accelerationIncludingGravity.x, expectedMotionData.accelerationIncludingGravityX, "accelerationIncludingGravity.x");
  assert_equals(event.accelerationIncludingGravity.y, expectedMotionData.accelerationIncludingGravityY, "accelerationIncludingGravity.y");
  assert_equals(event.accelerationIncludingGravity.z, expectedMotionData.accelerationIncludingGravityZ, "accelerationIncludingGravity.z");

  assert_approx_equals(event.rotationRate.alpha, expectedMotionData.rotationRateAlpha, MOTION_ROTATION_EPSILON, "rotationRate.alpha");
  assert_approx_equals(event.rotationRate.beta, expectedMotionData.rotationRateBeta, MOTION_ROTATION_EPSILON, "rotationRate.beta");
  assert_approx_equals(event.rotationRate.gamma, expectedMotionData.rotationRateGamma, MOTION_ROTATION_EPSILON, "rotationRate.gamma");

  assert_equals(event.interval, expectedMotionData.interval, "interval");
}

function checkOrientation(event, expectedOrientationData) {
  assert_equals(event.alpha, expectedOrientationData.alpha, "alpha");
  assert_equals(event.beta, expectedOrientationData.beta, "beta");
  assert_equals(event.gamma, expectedOrientationData.gamma, "gamma");

  assert_equals(event.absolute, expectedOrientationData.absolute, "absolute");
}

// Returns a promise that will be resolved when an event equal to the given
// event is fired.
function waitForEvent(expectedEvent, targetWindow = window) {
  const stringify = (thing, targetWindow) => {
    if (thing instanceof targetWindow.Object && thing.constructor !== targetWindow.Object) {
      let str = '{';
      for (let key of Object.keys(Object.getPrototypeOf(thing))) {
        str += JSON.stringify(key) + ': ' + stringify(thing[key], targetWindow) + ', ';
      }
      return str + '}';
    } else if (thing instanceof Number) {
      return thing.toFixed(6);
    }
    return JSON.stringify(thing);
  };

  return new Promise((resolve, reject) => {
    let events = [];
    let timeoutId = null;

    const expectedEventString = stringify(expectedEvent, window);
    function listener(event) {
      const eventString = stringify(event, targetWindow);
      if (eventString === expectedEventString) {
        targetWindow.clearTimeout(timeoutId);
        targetWindow.removeEventListener(expectedEvent.type, listener);
        resolve();
      } else {
        events.push(eventString);
      }
    }
    targetWindow.addEventListener(expectedEvent.type, listener);

    timeoutId = targetWindow.setTimeout(() => {
      targetWindow.removeEventListener(expectedEvent.type, listener);
      let errorMessage = 'Timeout waiting for expected event: ' + expectedEventString;
      if (events.length == 0) {
        errorMessage += ', no events were fired';
      } else {
        errorMessage += ', received events: '
        for (let event of events) {
          errorMessage += event + ', ';
        }
      }
      reject(errorMessage);
    }, 500);
  });
}

function waitForOrientation(expectedOrientationData, targetWindow = window) {
  return waitForEvent(
      new DeviceOrientationEvent('deviceorientation', {
        alpha: expectedOrientationData.alpha,
        beta: expectedOrientationData.beta,
        gamma: expectedOrientationData.gamma,
        absolute: expectedOrientationData.absolute,
      }),
      targetWindow);
}

function waitForAbsoluteOrientation(expectedOrientationData, targetWindow = window) {
  return waitForEvent(
      new DeviceOrientationEvent('deviceorientationabsolute', {
        alpha: expectedOrientationData.alpha,
        beta: expectedOrientationData.beta,
        gamma: expectedOrientationData.gamma,
        absolute: expectedOrientationData.absolute,
      }),
      targetWindow);
}

function waitForMotion(expectedMotionData, targetWindow = window) {
  return waitForEvent(
      new DeviceMotionEvent('devicemotion', {
        acceleration: {
          x: expectedMotionData.accelerationX,
          y: expectedMotionData.accelerationY,
          z: expectedMotionData.accelerationZ,
        },
        accelerationIncludingGravity: {
          x: expectedMotionData.accelerationIncludingGravityX,
          y: expectedMotionData.accelerationIncludingGravityY,
          z: expectedMotionData.accelerationIncludingGravityZ,
        },
        rotationRate: {
          alpha: expectedMotionData.rotationRateAlpha,
          beta: expectedMotionData.rotationRateBeta,
          gamma: expectedMotionData.rotationRateGamma,
        },
        interval: expectedMotionData.interval,
      }),
      targetWindow);
}
