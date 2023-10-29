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
function runGenericSensorTests(sensorData, readingData) {
  validate_sensor_data(sensorData);
  validate_reading_data(readingData);

  const {sensorName, permissionName, testDriverName, featurePolicyNames} =
      sensorData;
  const sensorType = self[sensorName];

  function sensor_test(func, name, properties) {
    promise_test(async t => {
      assert_implements(sensorName in self, `${sensorName} is not supported.`);

      const readings = new RingBuffer(readingData.readings);
      const expectedReadings = new RingBuffer(readingData.expectedReadings);
      const expectedRemappedReadings = readingData.expectedRemappedReadings ?
          new RingBuffer(readingData.expectedRemappedReadings) :
          undefined;

      return func(t, readings, expectedReadings, expectedRemappedReadings);
    }, name, properties);
  }

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'denied');

    await test_driver.create_virtual_sensor(testDriverName);
    const sensor = new sensorType;
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['reading', 'error']);
    sensor.start();

    const event = await sensorWatcher.wait_for('error');

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotAllowedError');
  }, `${sensorName}: Test that onerror is sent when permissions are not\
 granted.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName, {connected: false});
    const sensor = new sensorType;
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['reading', 'error']);

    sensor.start();

    const event = await sensorWatcher.wait_for('error');

    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotReadableError');
  }, `${sensorName}: Test that onerror is send when start() call has failed.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType({frequency: 560});
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');
    const mockSensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);

    assert_less_than_equal(mockSensorInfo.requestedSamplingFrequency, 60);
  }, `${sensorName}: Test that frequency is capped to allowed maximum.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    const maxSupportedFrequency = 5;
    await test_driver.create_virtual_sensor(
        testDriverName, {maxSamplingFrequency: maxSupportedFrequency});

    const sensor = new sensorType({frequency: 50});
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');
    const mockSensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);

    assert_equals(
        mockSensorInfo.requestedSamplingFrequency, maxSupportedFrequency);
  }, `${sensorName}: Test that frequency is capped to the maximum supported\
 frequency.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    const minSupportedFrequency = 2;
    await test_driver.create_virtual_sensor(
        testDriverName, {minSamplingFrequency: minSupportedFrequency});

    const sensor = new sensorType({frequency: -1});
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');
    const mockSensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);

    assert_equals(
        mockSensorInfo.requestedSamplingFrequency, minSupportedFrequency);
  }, `${sensorName}: Test that frequency is limited to the minimum supported\
 frequency.`);

  sensor_test(async t => {
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicyNames.join(' \'none\'; ') + ' \'none\';';
    iframe.srcdoc = '<script>' +
        '  window.onmessage = message => {' +
        '    if (message.data === "LOADED") {' +
        '      try {' +
        '        new ' + sensorName + '();' +
        '        parent.postMessage("FAIL", "*");' +
        '      } catch (e) {' +
        '        parent.postMessage(`PASS: got ${e.name}`, "*");' +
        '      }' +
        '    }' +
        '   };' +
        '<\/script>';
    const iframeWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeWatcher.wait_for('load');
    iframe.contentWindow.postMessage('LOADED', '*');

    const windowWatcher = new EventWatcher(t, window, 'message');
    const message = await windowWatcher.wait_for('message');
    assert_equals(message.data, 'PASS: got SecurityError');
  }, `${sensorName}: Test that sensor cannot be constructed within iframe\
 disallowed to use feature policy.`);

  sensor_test(async t => {
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicyNames.join(';') + ';';
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
    const iframeWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeWatcher.wait_for('load');
    iframe.contentWindow.postMessage('LOADED', '*');

    const windowWatcher = new EventWatcher(t, window, 'message');
    const message = await windowWatcher.wait_for('message');
    assert_equals(message.data, 'PASS');
  }, `${sensorName}: Test that sensor can be constructed within an iframe\
 allowed to use feature policy.`);

  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType;
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);

    sensor.start();
    assert_false(sensor.hasReading);
    await sensorWatcher.wait_for('activate');

    await Promise.all([
      test_driver.update_virtual_sensor(testDriverName, readings.next().value),
      sensorWatcher.wait_for('reading')
    ]);

    assert_sensor_reading_equals(sensor, expectedReadings.next().value);

    assert_true(sensor.hasReading);

    sensor.stop();

    assert_sensor_reading_is_null(sensor);
    assert_false(sensor.hasReading);
  }, `${sensorName}: Test that 'onreading' is called and sensor reading is\
 valid.`);

  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor1 = new sensorType();
    const sensor2 = new sensorType();
    t.add_cleanup(async () => {
      sensor1.stop();
      sensor2.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher1 =
        new EventWatcher(t, sensor1, ['activate', 'reading', 'error']);
    const sensorWatcher2 =
        new EventWatcher(t, sensor2, ['activate', 'reading', 'error']);
    sensor1.start();
    sensor2.start();

    await Promise.all([
      sensorWatcher1.wait_for('activate'), sensorWatcher2.wait_for('activate')
    ]);

    await Promise.all([
      test_driver.update_virtual_sensor(testDriverName, readings.next().value),
      sensorWatcher1.wait_for('reading'), sensorWatcher2.wait_for('reading')
    ]);

    // Reading values are correct for both sensors.
    const expected = expectedReadings.next().value;
    assert_sensor_reading_equals(sensor1, expected);
    assert_sensor_reading_equals(sensor2, expected);

    // After first sensor stops its reading values are null,
    // reading values for the second sensor sensor remain.
    sensor1.stop();
    assert_sensor_reading_is_null(sensor1);
    assert_sensor_reading_equals(sensor2, expected);

    sensor2.stop();
    assert_sensor_reading_is_null(sensor2);
  }, `${sensorName}: sensor reading is correct.`);

  // Tests that readings maps to expectedReadings correctly. Due to threshold
  // check and rounding some values might be discarded or changed.
  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');

    const sensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);
    const sensorPeriodInMs = (1 / sensorInfo.requestedSamplingFrequency) * 1000;

    for (let expectedReading of expectedReadings.data) {
      await update_virtual_sensor_until_reading(
          t, readings, sensorWatcher.wait_for('reading'), testDriverName,
          sensorPeriodInMs * 3);
      assert_true(sensor.hasReading, 'hasReading');
      assert_sensor_reading_equals(sensor, expectedReading);
    }
  }, `${sensorName}: Test that readings are all mapped to expectedReadings\
 correctly.`);

  sensor_test(async (t, readings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');

    const sensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);
    const sensorPeriodInMs = (1 / sensorInfo.requestedSamplingFrequency) * 1000;

    await Promise.all([
      test_driver.update_virtual_sensor(testDriverName, readings.next().value),
      sensorWatcher.wait_for('reading')
    ]);
    const cachedTimeStamp1 = sensor.timestamp;

    await update_virtual_sensor_until_reading(
        t, readings, sensorWatcher.wait_for('reading'), testDriverName,
        sensorPeriodInMs * 3);
    const cachedTimeStamp2 = sensor.timestamp;

    assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
  }, `${sensorName}: sensor timestamp is updated when time passes.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    assert_false(sensor.activated);
    sensor.start();
    assert_false(sensor.activated);

    await sensorWatcher.wait_for('activate');
    assert_true(sensor.activated);

    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: Test that sensor can be successfully created and its\
 states are correct.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    sensor.start();
    sensor.start();

    await sensorWatcher.wait_for('activate');
    assert_true(sensor.activated);
  }, `${sensorName}: no exception is thrown when calling start() on already\
 started sensor.`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['activate', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');
    sensor.stop();
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorName}: no exception is thrown when calling stop() on already\
 stopped sensor.`);

  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('activate');

    await Promise.all([
      test_driver.update_virtual_sensor(testDriverName, readings.next().value),
      sensorWatcher.wait_for('reading')
    ]);

    assert_true(sensor.hasReading);

    const expected = expectedReadings.next().value;
    assert_sensor_reading_equals(sensor, expected);

    const timestamp = sensor.timestamp;
    sensor.stop();
    assert_false(sensor.hasReading);
    assert_false(sensor.activated);

    sensor.start();

    await sensorWatcher.wait_for('activate');
    assert_false(sensor.hasReading);
    readings.reset();
    await Promise.all([
      test_driver.update_virtual_sensor(testDriverName, readings.next().value),
      sensorWatcher.wait_for('reading')
    ]);
    assert_true(sensor.hasReading);

    assert_sensor_reading_equals(sensor, expected);
    // Make sure that 'timestamp' is already initialized.
    assert_greater_than(timestamp, 0);
    // Check that the reading is updated.
    assert_greater_than(sensor.timestamp, timestamp);
  }, `${sensorName}: Test that fresh reading is fetched on start().`);

  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);

    sensor.start();
    await sensorWatcher.wait_for('activate');

    assert_false(sensor.hasReading);
    assert_sensor_reading_is_null(sensor);

    const {minimize, restore} = window_state_context(t);

    await minimize();
    assert_true(document.hidden);
    assert_true(sensor.activated);
    assert_false(sensor.hasReading);
    assert_sensor_reading_is_null(sensor);

    const reading = readings.next().value;
    await test_driver.update_virtual_sensor(testDriverName, reading);

    await restore();
    assert_false(document.hidden);
    assert_true(sensor.activated);
    assert_false(sensor.hasReading);
    assert_sensor_reading_is_null(sensor);

    const visiblePageTimestamp = performance.now();

    const [readingEvent] = await Promise.all([
      sensorWatcher.wait_for('reading'),
      test_driver.update_virtual_sensor(testDriverName, reading),
    ]);

    const postReadingTimestamp = performance.now();

    assert_sensor_reading_equals(sensor, expectedReadings.next().value);

    // Check that the only reading we received all this time was the one sent
    // after the page was made visible again. This is done by verifying the
    // timestamps of the event as well as the current reading's.
    assert_greater_than_equal(sensor.timestamp, visiblePageTimestamp);
    assert_greater_than(readingEvent.timeStamp, sensor.timestamp);
    assert_greater_than_equal(
        postReadingTimestamp, readingEvent.timeStamp,
        'No new reading events have been delivered');
  }, `${sensorName}: Readings are not delivered when the page has no visibility`);

  sensor_test(async t => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const fastSensor = new sensorType({frequency: 60});
    t.add_cleanup(() => {
      fastSensor.stop();
    });
    let eventWatcher = new EventWatcher(t, fastSensor, ['activate']);
    fastSensor.start();

    // Wait for |fastSensor| to be activated so that the call to
    // getSamplingFrequency() below works.
    await eventWatcher.wait_for('activate');

    let mockSensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);

    // We need |fastSensorFrequency| because 60Hz might be higher than a sensor
    // type's maximum allowed frequency.
    const fastSensorFrequency = mockSensorInfo.requestedSamplingFrequency;
    const slowSensorFrequency = fastSensorFrequency * 0.25;

    const slowSensor = new sensorType({frequency: slowSensorFrequency});
    t.add_cleanup(() => {
      slowSensor.stop();
    });
    t.add_cleanup(async () => {
      // Remove the virtual sensor only after calling stop() on both sensors.
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    eventWatcher = new EventWatcher(t, slowSensor, 'activate');
    slowSensor.start();

    // Wait for |slowSensor| to be activated before we check if the mock
    // platform sensor's sampling frequency has changed.
    await eventWatcher.wait_for('activate');
    mockSensorInfo =
        await test_driver.get_virtual_sensor_information(testDriverName);
    assert_equals(
        mockSensorInfo.requestedSamplingFrequency, fastSensorFrequency);

    // Now stop |fastSensor| and verify that the sampling frequency has dropped
    // to the one |slowSensor| had requested.
    fastSensor.stop();
    await wait_for_virtual_sensor_state(testDriverName, (info) => {
      return info.requestedSamplingFrequency === slowSensorFrequency;
    });
  }, `${sensorName}: frequency hint works.`);

  sensor_test(async (t, readings, expectedReadings) => {
    await test_driver.set_permission({name: permissionName}, 'granted');

    await test_driver.create_virtual_sensor(testDriverName);

    const sensor1 = new sensorType();
    const sensor2 = new sensorType();

    t.add_cleanup(async () => {
      sensor1.stop();
      sensor2.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });

    return new Promise(async (resolve, reject) => {
      sensor1.addEventListener('reading', () => {
        sensor2.addEventListener('activate', () => {
          try {
            assert_true(sensor1.activated);
            assert_true(sensor1.hasReading);

            const expected = expectedReadings.next().value;
            assert_sensor_reading_equals(sensor1, expected);

            assert_true(sensor2.activated);
            assert_sensor_reading_equals(sensor2, expected);
          } catch (e) {
            reject(e);
          }
        }, {once: true});
        sensor2.addEventListener('reading', () => {
          try {
            assert_true(sensor2.activated);
            assert_true(sensor2.hasReading);
            assert_sensor_reading_equals(sensor1, sensor2);
            assert_equals(sensor1.timestamp, sensor2.timestamp);
            resolve();
          } catch (e) {
            reject(e);
          }
        }, {once: true});
        sensor2.start();
      }, {once: true});

      const eventWatcher = new EventWatcher(t, sensor1, ['activate']);
      sensor1.start();
      await eventWatcher.wait_for('activate');
      test_driver.update_virtual_sensor(testDriverName, readings.next().value);
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
    const invalidFreqs = ['invalid', NaN, Infinity, -Infinity, {}];
    invalidFreqs.map(freq => {
      assert_throws_js(
          TypeError, () => {new sensorType({frequency: freq})},
          `when freq is ${freq}`);
    });
  }, `${sensorName}: throw 'TypeError' if frequency is invalid.`);

  if (!readingData.expectedRemappedReadings) {
    // The sensorType does not represent a spatial sensor.
    return;
  }

  // TODO(https://github.com/web-platform-tests/wpt/issues/42724): Re-enable
  // when there is a cross-platform way to set an orientation angle.
  // sensor_test(
  //     async (t, readings, expectedReadings, expectedRemappedReadings) => {
  //       assert_implements_optional(screen.orientation.angle == 270,
  //         'Remapped values expect a specific screen rotation.');
  //       await test_driver.set_permission({name: permissionName}, 'granted');

  //       await test_driver.create_virtual_sensor(testDriverName);

  //       const sensor1 = new sensorType({frequency: 60});
  //       const sensor2 =
  //           new sensorType({frequency: 60, referenceFrame: 'screen'});
  //       t.add_cleanup(async () => {
  //         sensor1.stop();
  //         sensor2.stop();
  //         await test_driver.remove_virtual_sensor(testDriverName);
  //       });
  //       const sensorWatcher1 =
  //           new EventWatcher(t, sensor1, ['activate', 'reading', 'error']);
  //       const sensorWatcher2 =
  //           new EventWatcher(t, sensor1, ['activate', 'reading', 'error']);

  //       sensor1.start();
  //       sensor2.start();

  //       await Promise.all([
  //         sensorWatcher1.wait_for('activate'),
  //         sensorWatcher2.wait_for('activate')
  //       ]);

  //       await Promise.all([
  //         test_driver.update_virtual_sensor(testDriverName,
  //         readings.next().value), sensorWatcher1.wait_for('reading'),
  //         sensorWatcher2.wait_for('reading')
  //       ]);

  //       const expected = expectedReadings.next().value;
  //       const expectedRemapped = expectedRemappedReadings.next().value;
  //       assert_sensor_reading_equals(sensor1, expected);
  //       assert_sensor_reading_equals(sensor2, expectedRemapped);

  //       sensor1.stop();
  //       assert_sensor_reading_is_null(sensor1);
  //       assert_sensor_reading_equals(sensor2, expectedRemapped);

  //       sensor2.stop();
  //       assert_sensor_reading_is_null(sensor2);
  //     },
  //     `${sensorName}: sensor reading is correct when options.referenceFrame\
  // is 'screen'.`);
}

function runGenericSensorInsecureContext(sensorName) {
  test(() => {
    assert_false(sensorName in window, `${sensorName} must not be exposed`);
  }, `${sensorName} is not exposed in an insecure context.`);
}
