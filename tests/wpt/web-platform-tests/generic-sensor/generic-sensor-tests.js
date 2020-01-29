'use strict';

// Run a set of tests for a given |sensorName|.
// |readingData| is an object with 3 keys, all of which are arrays of arrays:
// 1. "readings". Each value corresponds to one raw reading that will be
//    processed by a sensor.
// 2. "expectedReadings". Each value corresponds to the processed value a
//    sensor will make available to users (i.e. a capped or rounded value).
//    Its length must match |readings|'.
// 3. "expectedRemappedReadings" (optional). Similar to |expectedReadings|, but
//    used only by spatial sensors, whose reference frame can change the values
//    returned by a sensor.
//    Its length should match |readings|'.
// |verificationFunction| is called to verify that a given reading matches a
// value in |expectedReadings|.
// |featurePolicies| represents |sensorName|'s associated sensor feature name.

function runGenericSensorTests(sensorName,
                               readingData,
                               verificationFunction,
                               featurePolicies) {
  const sensorType = self[sensorName];

  function validateReadingFormat(data) {
    return Array.isArray(data) && data.every(element => Array.isArray(element));
  }

  const { readings, expectedReadings, expectedRemappedReadings } = readingData;
  if (!validateReadingFormat(readings)) {
    throw new TypeError('readingData.readings must be an array of arrays.');
  }
  if (!validateReadingFormat(expectedReadings)) {
    throw new TypeError('readingData.expectedReadings must be an array of ' +
                        'arrays.');
  }
  if (readings.length != expectedReadings.length) {
    throw new TypeError('readingData.readings and ' +
                        'readingData.expectedReadings must have the same ' +
                        'length.');
  }
  if (expectedRemappedReadings &&
      !validateReadingFormat(expectedRemappedReadings)) {
    throw new TypeError('readingData.expectedRemappedReadings must be an ' +
                        'array of arrays.');
  }
  if (expectedRemappedReadings &&
      readings.length != expectedRemappedReadings.length) {
    throw new TypeError('readingData.readings and ' +
      'readingData.expectedRemappedReadings must have the same ' +
      'length.');
  }

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    sensorProvider.setGetSensorShouldFail(sensorName, true);
    const sensor = new sensorType;
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const event = await sensorWatcher.wait_for("error");

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotReadableError');
  }, `${sensorName}: Test that onerror is sent when sensor is not supported.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    sensorProvider.setPermissionsDenied(sensorName, true);
    const sensor = new sensorType;
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const event = await sensorWatcher.wait_for("error");

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotAllowedError');
  }, `${sensorName}: Test that onerror is sent when permissions are not\
 granted.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor = new sensorType({frequency: 560});
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setStartShouldFail(true);

    const event = await sensorWatcher.wait_for("error");

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotReadableError');
  }, `${sensorName}: Test that onerror is send when start() call has failed.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor = new sensorType({frequency: 560});
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);

    await sensorWatcher.wait_for("activate");

    assert_less_than_equal(mockSensor.getSamplingFrequency(), 60);
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: Test that frequency is capped to allowed maximum.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const maxSupportedFrequency = 5;
    sensorProvider.setMaximumSupportedFrequency(maxSupportedFrequency);
    const sensor = new sensorType({frequency: 50});
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);

    await sensorWatcher.wait_for("activate");

    assert_equals(mockSensor.getSamplingFrequency(), maxSupportedFrequency);
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: Test that frequency is capped to the maximum supported\
 frequency.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const minSupportedFrequency = 2;
    sensorProvider.setMinimumSupportedFrequency(minSupportedFrequency);
    const sensor = new sensorType({frequency: -1});
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);

    await sensorWatcher.wait_for("activate");

    assert_equals(mockSensor.getSamplingFrequency(), minSupportedFrequency);
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: Test that frequency is limited to the minimum supported\
 frequency.`);

  promise_test(async t => {
    assert_true(sensorName in self);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(' \'none\'; ') + ' \'none\';';
    iframe.srcdoc = '<script>' +
                    '  window.onmessage = message => {' +
                    '    if (message.data === "LOADED") {' +
                    '      try {' +
                    '        new ' + sensorName + '();' +
                    '        parent.postMessage("FAIL", "*");' +
                    '      } catch (e) {' +
                    '        parent.postMessage("PASS", "*");' +
                    '      }' +
                    '    }' +
                    '   };' +
                    '<\/script>';
    const iframeWatcher = new EventWatcher(t, iframe, "load");
    document.body.appendChild(iframe);
    await iframeWatcher.wait_for("load");
    iframe.contentWindow.postMessage('LOADED', '*');

    const windowWatcher = new EventWatcher(t, window, "message");
    const message = await windowWatcher.wait_for("message");
    assert_equals(message.data, 'PASS');
  }, `${sensorName}: Test that sensor cannot be constructed within iframe\
 disallowed to use feature policy.`);

  promise_test(async t => {
    assert_true(sensorName in self);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.srcdoc = '<script>' +
                    '  window.onmessage = message => {' +
                    '    if (message.data === "LOADED") {' +
                    '      try {' +
                    '        new ' + sensorName + '();' +
                    '        parent.postMessage("PASS", "*");' +
                    '      } catch (e) {' +
                    '        parent.postMessage("FAIL", "*");' +
                    '      }' +
                    '    }' +
                    '   };' +
                    '<\/script>';
    const iframeWatcher = new EventWatcher(t, iframe, "load");
    document.body.appendChild(iframe);
    await iframeWatcher.wait_for("load");
    iframe.contentWindow.postMessage('LOADED', '*');

    const windowWatcher = new EventWatcher(t, window, "message");
    const message = await windowWatcher.wait_for("message");
    assert_equals(message.data, 'PASS');
  }, `${sensorName}: Test that sensor can be constructed within an iframe\
 allowed to use feature policy.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();
    assert_false(sensor.hasReading);

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");
    const expected = new RingBuffer(expectedReadings).next().value;
    assert_true(verificationFunction(expected, sensor));
    assert_true(sensor.hasReading);

    sensor.stop();
    assert_true(verificationFunction(expected, sensor, /*isNull=*/true));
    assert_false(sensor.hasReading);
  }, `${sensorName}: Test that 'onreading' is called and sensor reading is\
 valid.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor1 = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor1, ["reading", "error"]);
    sensor1.start();

    const sensor2 = new sensorType();
    sensor2.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");
    const expected = new RingBuffer(expectedReadings).next().value;
    // Reading values are correct for both sensors.
    assert_true(verificationFunction(expected, sensor1));
    assert_true(verificationFunction(expected, sensor2));

    // After first sensor stops its reading values are null,
    // reading values for the second sensor sensor remain.
    sensor1.stop();
    assert_true(verificationFunction(expected, sensor1, /*isNull=*/true));
    assert_true(verificationFunction(expected, sensor2));

    sensor2.stop();
    assert_true(verificationFunction(expected, sensor2, /*isNull=*/true));
  }, `${sensorName}: sensor reading is correct.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp1 = sensor.timestamp;

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp2 = sensor.timestamp;

    assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
    sensor.stop();
  }, `${sensorName}: sensor timestamp is updated when time passes.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    assert_false(sensor.activated);
    sensor.start();
    assert_false(sensor.activated);

    await sensorWatcher.wait_for("activate");
    assert_true(sensor.activated);

    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: Test that sensor can be successfully created and its\
 states are correct.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    const start_return = sensor.start();

    await sensorWatcher.wait_for("activate");
    assert_equals(start_return, undefined);
    sensor.stop();
  }, `${sensorName}: sensor.start() returns undefined.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();
    sensor.start();

    await sensorWatcher.wait_for("activate");
    assert_true(sensor.activated);
    sensor.stop();
  }, `${sensorName}: no exception is thrown when calling start() on already\
 started sensor.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("activate");
    const stop_return = sensor.stop();
    assert_equals(stop_return, undefined);
  }, `${sensorName}: sensor.stop() returns undefined.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("activate");
    sensor.stop();
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: no exception is thrown when calling stop() on already\
 stopped sensor.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    const expectedBuffer = new RingBuffer(expectedReadings);
    await sensorWatcher.wait_for("reading");
    const expected1 = expectedBuffer.next().value;
    assert_true(sensor.hasReading);
    assert_true(verificationFunction(expected1, sensor));
    const timestamp = sensor.timestamp;
    sensor.stop();
    assert_false(sensor.hasReading);

    sensor.start();
    await sensorWatcher.wait_for("reading");
    assert_true(sensor.hasReading);
    // |readingData| may have a single reading/expectation value, and this
    // is the second reading we are getting. For that case, make sure we
    // also wrap around as if we had the same RingBuffer used in
    // generic_sensor_mocks.js.
    const expected2 = expectedBuffer.next().value;
    assert_true(verificationFunction(expected2, sensor));
    // Make sure that 'timestamp' is already initialized.
    assert_greater_than(timestamp, 0);
    // Check that the reading is updated.
    assert_greater_than(sensor.timestamp, timestamp);
    sensor.stop();
  }, `${sensorName}: Test that fresh reading is fetched on start().`);

//  TBD file a WPT issue: visibilityChangeWatcher times out.
//  sensor_test(async (t, sensorProvider) => {
//    assert_true(sensorName in self);
//    const sensor = new sensorType();
//    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
//    const visibilityChangeWatcher = new EventWatcher(t, document,
//                                                     "visibilitychange");
//    sensor.start();

//    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
//    await mockSensor.setSensorReading(readings);

//    await sensorWatcher.wait_for("reading");
//    const expected = new RingBuffer(expectedReadings).next().value;
//    assert_true(verificationFunction(expected, sensor));
//    const cachedTimestamp1 = sensor.timestamp;

//    const win = window.open('', '_blank');
//    await visibilityChangeWatcher.wait_for("visibilitychange");
//    const cachedTimestamp2 = sensor.timestamp;

//    win.close();
//    sensor.stop();
//    assert_equals(cachedTimestamp1, cachedTimestamp2);
//  }, `${sensorName}: sensor readings can not be fired on the background tab.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const fastSensor = new sensorType({frequency: 60});
    fastSensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    const fastCounter = await new Promise((resolve, reject) => {
      let fastSensorNotifiedCounter = 0;
      let slowSensorNotifiedCounter = 0;

      fastSensor.onreading = () => {
        if (fastSensorNotifiedCounter === 0) {
          // For Magnetometer and ALS, the maximum frequency is less than 60Hz
          // we make "slow" sensor 4 times slower than the actual applied
          // frequency, so that the "fast" sensor will immediately overtake it
          // despite the notification adjustments.
          const slowFrequency = mockSensor.getSamplingFrequency() * 0.25;
          const slowSensor = new sensorType({frequency: slowFrequency});
          slowSensor.onreading = () => {
            // Skip the initial notification that always comes immediately.
            if (slowSensorNotifiedCounter === 1) {
              fastSensor.stop();
              slowSensor.stop();
              resolve(fastSensorNotifiedCounter);
            }
            slowSensorNotifiedCounter++;
          }
          slowSensor.onerror = reject;
          slowSensor.start();
        }
        fastSensorNotifiedCounter++;
      }
      fastSensor.onerror = reject;
    });
    assert_greater_than(fastCounter, 2, "Fast sensor overtakes the slow one");
  }, `${sensorName}: frequency hint works.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    // Create a focused editbox inside a cross-origin iframe,
    // sensor notification must suspend.
    const iframeSrc = 'data:text/html;charset=utf-8,<html><body>'
                    + '<input type="text" autofocus></body></html>';
    const iframe = document.createElement('iframe');
    iframe.src = encodeURI(iframeSrc);

    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");
    const expected = new RingBuffer(expectedReadings).next().value;
    assert_true(verificationFunction(expected, sensor));
    const cachedTimestamp1 = sensor.timestamp;

    const iframeWatcher = new EventWatcher(t, iframe, "load");
    document.body.appendChild(iframe);
    await iframeWatcher.wait_for("load");
    const cachedTimestamp2 = sensor.timestamp;
    assert_equals(cachedTimestamp1, cachedTimestamp2);

    iframe.remove();
    await sensorWatcher.wait_for("reading");
    assert_greater_than(sensor.timestamp, cachedTimestamp1);

    sensor.stop();
  }, `${sensorName}: sensor receives suspend / resume notifications when\
 cross-origin subframe is focused.`);

//  Re-enable after https://github.com/w3c/sensors/issues/361 is fixed.
//  test(() => {
//     assert_throws_dom("NotSupportedError",
//         () => { new sensorType({invalid: 1}) });
//     assert_throws_dom("NotSupportedError",
//         () => { new sensorType({frequency: 60, invalid: 1}) });
//     if (!expectedRemappedReadings) {
//       assert_throws_dom("NotSupportedError",
//           () => { new sensorType({referenceFrame: "screen"}) });
//     }
//  }, `${sensorName}: throw 'NotSupportedError' for an unsupported sensor\
// option.`);

  test(() => {
    assert_true(sensorName in self);
    const invalidFreqs = [
      "invalid",
      NaN,
      Infinity,
      -Infinity,
      {}
    ];
    invalidFreqs.map(freq => {
      assert_throws_js(TypeError,
                       () => { new sensorType({frequency: freq}) },
                       `when freq is ${freq}`);
    });
  }, `${sensorName}: throw 'TypeError' if frequency is invalid.`);

  if (!expectedRemappedReadings) {
    // The sensorType does not represent a spatial sensor.
    return;
  }

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    const sensor1 = new sensorType({frequency: 60});
    const sensor2 = new sensorType({frequency: 60, referenceFrame: "screen"});
    const sensorWatcher = new EventWatcher(t, sensor1, ["reading", "error"]);

    sensor1.start();
    sensor2.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");

    const expected = new RingBuffer(expectedReadings).next().value;
    const expectedRemapped =
        new RingBuffer(expectedRemappedReadings).next().value;
    assert_true(verificationFunction(expected, sensor1));
    assert_true(verificationFunction(expectedRemapped, sensor2));

    sensor1.stop();
    assert_true(verificationFunction(expected, sensor1, /*isNull=*/true));
    assert_true(verificationFunction(expectedRemapped, sensor2));

    sensor2.stop();
    assert_true(verificationFunction(expectedRemapped, sensor2,
                                     /*isNull=*/true));
  }, `${sensorName}: sensor reading is correct when options.referenceFrame\
 is 'screen'.`);

  test(() => {
    assert_true(sensorName in self);
    const invalidRefFrames = [
      "invalid",
      null,
      123,
      {},
      "",
      true
    ];
    invalidRefFrames.map(refFrame => {
      assert_throws_js(TypeError,
                       () => { new sensorType({referenceFrame: refFrame}) },
                       `when refFrame is ${refFrame}`);
    });
  }, `${sensorName}: throw 'TypeError' if referenceFrame is not one of\
 enumeration values.`);
}

function runGenericSensorInsecureContext(sensorName) {
  test(() => {
    assert_false(sensorName in window, `${sensorName} must not be exposed`);
  }, `${sensorName} is not exposed in an insecure context.`);
}
