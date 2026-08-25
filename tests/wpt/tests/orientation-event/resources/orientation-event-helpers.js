'use strict';

// @class SensorTestHelper
//
// SensorTestHelper is a helper utilities for orientation event tests.
//
// Usage example with device orientation:
//   const helper = new SensorTestHelper(t, 'deviceorientation');
//   await helper.grantSensorsPermissions();
//   await helper.initializeSensors();
//   const generatedData = generateOrientationData(1, 2, 3, false);
//   await helper.setData(generatedData);
//   await waitForEvent(getExpectedOrientationEvent(generatedData));
class SensorTestHelper {
  #eventName;
  #sensorsEnabledByDefault;
  #enabledSensors;
  #disabledSensors;
  #testObject;

  // @param {object} t - A testharness.js subtest instance.
  // @param {string} eventName - A name of event. Accepted values are
  //                             devicemotion, deviceorientation or
  //                             deviceorientationabsolute.
  constructor(t, eventName) {
    this.#eventName = eventName;
    this.#testObject = t;
    this.#testObject.add_cleanup(() => this.reset());

    switch (this.#eventName) {
      case 'devicemotion':
        this.#sensorsEnabledByDefault =
            new Set(['accelerometer', 'gyroscope', 'linear-acceleration']);
        break;
      case 'deviceorientation':
        this.#sensorsEnabledByDefault = new Set(['relative-orientation']);
        break;
      case 'deviceorientationabsolute':
        this.#sensorsEnabledByDefault = new Set(['absolute-orientation']);
        break;
      default:
        throw new Error(`Invalid event name ${this.#eventName}`);
    }
  }

  // Creates virtual sensors that will be used in tests.
  //
  // This function must be called before event listeners are added or calls
  // to setData() or waitForEvent() are made.
  //
  // The |options| parameter is an object that accepts the following entries:
  // - enabledSensors: A list of virtual sensor names that will be created
  //                   instead of the default ones for a given event type.
  // - disabledSensors: A list of virtual sensor names that will be created
  //                    in a disabled state, so that creating a sensor of
  //                    a given type is guaranteed to fail.
  // An Error is thrown if the same name is passed to both options.
  //
  // A default list of virtual sensors based on the |eventName| parameter passed
  // to the constructor is used if |options| is not specified.
  //
  // Usage examples
  // Use default sensors for the given event type:
  //   await helper.initializeSensors()
  // Enable specific sensors:
  //   await helper.initializeSensors({
  //     enabledSensors: ['accelerometer', 'gyroscope']
  //   })
  // Disable some sensors, make some report as not available:
  //   await helper.initializeSensors({
  //     disabledSensors: ['gyroscope']
  //   })
  // Enable some sensors, make some report as not available:
  //   await helper.initializeSensors({
  //     enabledSensors: ['accelerometer'],
  //     disabledSensors: ['gyroscope']
  //   })
  async initializeSensors(options = {}) {
    this.#disabledSensors = new Set(options.disabledSensors || []);
    // Check that a sensor name is not in both |options.enabledSensors| and
    // |options.disabledSensors|.
    for (const sensor of (options.enabledSensors || [])) {
      if (this.#disabledSensors.has(sensor)) {
        throw new Error(`${sensor} can be defined only as enabledSensors or disabledSensors`);
      }
    }

    this.#enabledSensors = new Set(options.enabledSensors || this.#sensorsEnabledByDefault);
    // Remove sensors from enabledSensors that are in disabledSensors
    for (const sensor of this.#disabledSensors) {
      this.#enabledSensors.delete(sensor);
    }

    const createVirtualSensorPromises = [];
    for (const sensor of this.#enabledSensors) {
      createVirtualSensorPromises.push(
          test_driver.create_virtual_sensor(sensor));
    }
    for (const sensor of this.#disabledSensors) {
      createVirtualSensorPromises.push(
          test_driver.create_virtual_sensor(sensor, {connected: false}));
    }
    await Promise.all(createVirtualSensorPromises);
  }

  // Updates virtual sensor with given data.
  // @param {object} data - Generated data by generateMotionData or
  //                        generateOrientationData which is passed to
  //                        test_driver.update_virtual_sensor().
  async setData(data) {
    // WebDriver expects numbers for all values in the readings it receives. We
    // convert null to zero here, but any other numeric value would work, as it
    // is the presence of one or more sensors in initializeSensors()'
    // options.disabledSensors that cause null to be reported in one or more
    // event attributes.
    const nullToZero = x => (x === null ? 0 : x);
    if (this.#eventName === 'devicemotion') {
      const degToRad = Math.PI / 180;
      await Promise.all([
        test_driver.update_virtual_sensor('accelerometer', {
          'x': nullToZero(data.accelerationIncludingGravityX),
          'y': nullToZero(data.accelerationIncludingGravityY),
          'z': nullToZero(data.accelerationIncludingGravityZ),
        }),
        test_driver.update_virtual_sensor('linear-acceleration', {
          'x': nullToZero(data.accelerationX),
          'y': nullToZero(data.accelerationY),
          'z': nullToZero(data.accelerationZ),
        }),
        test_driver.update_virtual_sensor('gyroscope', {
          'x': nullToZero(data.rotationRateAlpha) * degToRad,
          'y': nullToZero(data.rotationRateBeta) * degToRad,
          'z': nullToZero(data.rotationRateGamma) * degToRad,
        }),
      ]);
    } else {
      const sensorType =
          data.absolute ? 'absolute-orientation' : 'relative-orientation';
      await test_driver.update_virtual_sensor(sensorType, {
        alpha: nullToZero(data.alpha),
        beta: nullToZero(data.beta),
        gamma: nullToZero(data.gamma),
      });
    }
  }

  // Grants permissions to sensors. Depending on |eventName|, requests
  // permission to use either the DeviceMotionEvent or the
  // DeviceOrientationEvent API.
  async grantSensorsPermissions() {
    // Required by all event types.
    await test_driver.set_permission({name: 'accelerometer'}, 'granted');
    await test_driver.set_permission({name: 'gyroscope'}, 'granted');
    if (this.#eventName == 'deviceorientationabsolute') {
      await test_driver.set_permission({name: 'magnetometer'}, 'granted');
    }

    const interfaceName = this.#eventName == 'devicemotion' ?
        DeviceMotionEvent :
        DeviceOrientationEvent;
    await test_driver.bless('enable user activation', async () => {
      const permission = await interfaceName.requestPermission();
      assert_equals(permission, 'granted');
    });
  }

  // Resets SensorTestHelper to default state. Removes all created virtual
  // sensors.
  async reset() {
    const createdVirtualSensors =
      new Set([...this.#enabledSensors, ...this.#disabledSensors]);

    const sensorRemovalPromises = [];
    for (const sensor of createdVirtualSensors) {
      sensorRemovalPromises.push(test_driver.remove_virtual_sensor(sensor));
    }
    await Promise.all(sensorRemovalPromises);
  }
}

function generateMotionData(
    accelerationX, accelerationY, accelerationZ, accelerationIncludingGravityX,
    accelerationIncludingGravityY, accelerationIncludingGravityZ,
    rotationRateAlpha, rotationRateBeta, rotationRateGamma, interval = 16) {
  const motionData = {
    accelerationX: accelerationX,
    accelerationY: accelerationY,
    accelerationZ: accelerationZ,
    accelerationIncludingGravityX: accelerationIncludingGravityX,
    accelerationIncludingGravityY: accelerationIncludingGravityY,
    accelerationIncludingGravityZ: accelerationIncludingGravityZ,
    rotationRateAlpha: rotationRateAlpha,
    rotationRateBeta: rotationRateBeta,
    rotationRateGamma: rotationRateGamma,
    interval: interval
  };
  return motionData;
}

function generateOrientationData(alpha, beta, gamma, absolute) {
  const orientationData =
      {alpha: alpha, beta: beta, gamma: gamma, absolute: absolute};
  return orientationData;
}

function assertValueIsCoarsened(value) {
  // Checks that the precision of the value is at most 0.1.
  // https://www.w3.org/TR/orientation-event/ specification defines that all
  // measurements are required to be coarsened to 0.1 degrees, 0.1 m/s^2 or
  // 0.1 deg/s.
  const resolution = 0.1;
  const coarsenedValue = Math.round(value / resolution) * resolution;
  assert_approx_equals(value, coarsenedValue, Number.EPSILON,
                       `Expected ${value}'s precision to be at most ${resolution}`);
}

function assertEventEquals(actualEvent, expectedEvent) {
  // If two doubles differ by less than this amount, we can consider them
  // to be effectively equal.
  const EPSILON = 1e-8;

  for (let key1 of Object.keys(Object.getPrototypeOf(expectedEvent))) {
    if (typeof expectedEvent[key1] === 'object' &&
        expectedEvent[key1] !== null) {
      assertEventEquals(actualEvent[key1], expectedEvent[key1]);
    } else if (typeof expectedEvent[key1] === 'number') {
      assert_approx_equals(
          actualEvent[key1], expectedEvent[key1], EPSILON, key1);
    } else {
      assert_equals(actualEvent[key1], expectedEvent[key1], key1);
    }
  }
}

function getExpectedOrientationEvent(expectedOrientationData) {
  return new DeviceOrientationEvent('deviceorientation', {
    alpha: expectedOrientationData.alpha,
    beta: expectedOrientationData.beta,
    gamma: expectedOrientationData.gamma,
    absolute: expectedOrientationData.absolute,
  });
}

function getExpectedAbsoluteOrientationEvent(expectedOrientationData) {
  return new DeviceOrientationEvent('deviceorientationabsolute', {
    alpha: expectedOrientationData.alpha,
    beta: expectedOrientationData.beta,
    gamma: expectedOrientationData.gamma,
    absolute: expectedOrientationData.absolute,
  });
}

function getExpectedMotionEvent(expectedMotionData) {
  return new DeviceMotionEvent('devicemotion', {
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
  });
}

function waitForEvent(expected_event) {
  return new Promise((resolve, reject) => {
    window.addEventListener(expected_event.type, (event) => {
      try {
        assertEventEquals(event, expected_event);
        resolve();
      } catch (e) {
        reject(e);
      }
    }, {once: true});
  });
}
