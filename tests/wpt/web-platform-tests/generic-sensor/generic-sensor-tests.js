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
  if (readings.length < expectedReadings.length) {
    throw new TypeError('readingData.readings\' length must be bigger than ' +
                        'or equal to readingData.expectedReadings\' length.');
  }
  if (expectedRemappedReadings &&
      !validateReadingFormat(expectedRemappedReadings)) {
    throw new TypeError('readingData.expectedRemappedReadings must be an ' +
                        'array of arrays.');
  }
  if (expectedRemappedReadings &&
      expectedReadings.length != expectedRemappedReadings.length) {
    throw new TypeError('readingData.expectedReadings and ' +
      'readingData.expectedRemappedReadings must have the same ' +
      'length.');
  }

  sensor_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    sensorProvider.setGetSensorShouldFail(sensorName, true);
    const sensor = new sensorType;
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const event = await sensorWatcher.wait_for("error");

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotReadableError');
  }, `${sensorName}: Test that onerror is sent when sensor is not supported.`);

  sensor_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();
    assert_false(sensor.hasReading);

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor1 = new sensorType();
    const sensorWatcher1 = new EventWatcher(t, sensor1, ["reading", "error"]);
    sensor1.start();

    const sensor2 = new sensorType();
    const sensorWatcher2 = new EventWatcher(t, sensor2, ["reading", "error"]);
    sensor2.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

    await Promise.all([sensorWatcher1.wait_for("reading"),
                       sensorWatcher2.wait_for("reading")]);
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

  // Tests that readings maps to expectedReadings correctly. Due to threshold
  // check and rounding some values might be discarded or changed.
  sensor_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    await mockSensor.setSensorReading(readings);

    for (let expectedReading of expectedReadings) {
      await sensorWatcher.wait_for("reading");
      assert_true(sensor.hasReading, "hasReading");
      assert_true(verificationFunction(expectedReading, sensor),
                                       "verification");
    }

    sensor.stop();
  }, `${sensorName}: Test that readings are all mapped to expectedReadings\
 correctly.`);

  sensor_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp1 = sensor.timestamp;

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp2 = sensor.timestamp;

    assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
    sensor.stop();
  }, `${sensorName}: sensor timestamp is updated when time passes.`);

  sensor_test(async t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

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
//    assert_implements(sensorName in self, `${sensorName} is not supported.`);
//    const sensor = new sensorType();
//    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
//    const visibilityChangeWatcher = new EventWatcher(t, document,
//                                                     "visibilitychange");
//    sensor.start();

//    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
//    mockSensor.setSensorReading(readings);

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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);

    const fastSensor = new sensorType({ frequency: 60 });
    t.add_cleanup(() => { fastSensor.stop(); });
    let eventWatcher = new EventWatcher(t, fastSensor, "activate");
    fastSensor.start();

    // Wait for |fastSensor| to be activated so that the call to
    // getSamplingFrequency() below works.
    await eventWatcher.wait_for("activate");

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

    // We need |fastSensorFrequency| because 60Hz might be higher than a sensor
    // type's maximum allowed frequency.
    const fastSensorFrequency = mockSensor.getSamplingFrequency();
    const slowSensorFrequency = fastSensorFrequency * 0.25;

    const slowSensor = new sensorType({ frequency: slowSensorFrequency });
    t.add_cleanup(() => { slowSensor.stop(); });
    eventWatcher = new EventWatcher(t, slowSensor, "activate");
    slowSensor.start();

    // Wait for |slowSensor| to be activated before we check if the mock
    // platform sensor's sampling frequency has changed.
    await eventWatcher.wait_for("activate");
    assert_equals(mockSensor.getSamplingFrequency(), fastSensorFrequency);

    // Now stop |fastSensor| and verify that the sampling frequency has dropped
    // to the one |slowSensor| had requested.
    fastSensor.stop();
    return t.step_wait(() => {
      return mockSensor.getSamplingFrequency() === slowSensorFrequency;
    }, "Sampling frequency has dropped to slowSensor's requested frequency");
  }, `${sensorName}: frequency hint works.`);

  sensor_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);

    const sensor1 = new sensorType();
    const sensor2 = new sensorType();

    return new Promise((resolve, reject) => {
      sensor1.addEventListener('reading', () => {
        sensor2.addEventListener('activate', () => {
          try {
            assert_true(sensor1.activated);
            assert_true(sensor1.hasReading);
            assert_false(verificationFunction(null, sensor1, /*isNull=*/true));
            assert_not_equals(sensor1.timestamp, null);

            assert_true(sensor2.activated);
            assert_false(verificationFunction(null, sensor2, /*isNull=*/true));
            assert_not_equals(sensor2.timestamp, null);
          } catch (e) {
            reject(e);
          }
        }, { once: true });
        sensor2.addEventListener('reading', () => {
          try {
            assert_true(sensor2.activated);
            assert_true(sensor2.hasReading);
            assert_sensor_equals(sensor1, sensor2);
            resolve();
          } catch (e) {
            reject(e);
          }
        }, { once: true });
        sensor2.start();
      }, { once: true });
      sensor1.start();
    });
  }, `${sensorName}: Readings delivered by shared platform sensor are\
 immediately accessible to all sensors.`);

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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const sensor1 = new sensorType({frequency: 60});
    const sensor2 = new sensorType({frequency: 60, referenceFrame: "screen"});
    const sensorWatcher1 = new EventWatcher(t, sensor1, ["reading", "error"]);
    const sensorWatcher2 = new EventWatcher(t, sensor1, ["reading", "error"]);

    sensor1.start();
    sensor2.start();

    const mockSensor = await sensorProvider.getCreatedSensor(sensorName);
    mockSensor.setSensorReading(readings);

    await Promise.all([sensorWatcher1.wait_for("reading"),
                       sensorWatcher2.wait_for("reading")]);

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
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
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
