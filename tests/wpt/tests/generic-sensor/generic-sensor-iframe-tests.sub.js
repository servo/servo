function send_message_to_iframe(iframe, message) {
  return new Promise((resolve, reject) => {
    window.addEventListener('message', (e) => {
      // The usage of test_driver.set_test_context() in
      // iframe_sensor_handler.html causes unrelated messages to be sent as
      // well. We just need to ignore them here.
      if (!e.data.command) {
        return;
      }

      if (e.data.command !== message.command) {
        reject(`Expected reply with command '${message.command}', got '${
            e.data.command}' instead`);
        return;
      }
      if (e.data.error) {
        reject(e.data.error);
        return;
      }
      resolve(e.data.result);
    });
    iframe.contentWindow.postMessage(message, '*');
  });
}

function run_generic_sensor_iframe_tests(sensorData, readingData) {
  validate_sensor_data(sensorData);
  validate_reading_data(readingData);

  const {sensorName, permissionName, testDriverName} = sensorData;
  const sensorType = self[sensorName];
  const featurePolicies = get_feature_policies_for_sensor(sensorName);

  // When comparing timestamps in the tests below, we need to account for small
  // deviations coming from the way time is coarsened according to the High
  // Resolution Time specification, even more so when we need to translate
  // timestamps from different documents with different time origins.
  // 0.5 is 500 microseconds, which is acceptable enough given that even a high
  // sensor frequency beyond what is usually allowed like 100Hz has a period
  // much larger than 0.5ms.
  const ALLOWED_JITTER_IN_MS = 0.5;

  function sensor_test(func, name, properties) {
    promise_test(async t => {
      assert_implements(sensorName in self, `${sensorName} is not supported.`);
      const readings = new RingBuffer(readingData.readings);
      return func(t, readings);
    }, name, properties);
  }

  sensor_test(async (t, readings) => {
    // This is a specialized EventWatcher that works with a sensor inside a
    // cross-origin iframe. We cannot manipulate the sensor object there
    // directly from this frame, so we need the iframe to send us a message
    // when the "reading" event is fired, and we decide whether we were
    // expecting for it or not. This should be instantiated early in the test
    // to catch as many unexpected events as possible.
    class IframeSensorReadingEventWatcher {
      constructor(test_obj) {
        this.resolve_ = null;

        window.onmessage = test_obj.step_func((ev) => {
          // Unrelated message, ignore.
          if (!ev.data.eventName) {
            return;
          }

          assert_equals(
              ev.data.eventName, 'reading', 'Expecting a "reading" event');
          assert_true(
              !!this.resolve_,
              'Received "reading" event from iframe but was not expecting one');
          const resolveFunc = this.resolve_;
          this.resolve_ = null;
          resolveFunc(ev.data.serializedSensor);
        });
      }

      wait_for_reading() {
        return new Promise(resolve => {
          this.resolve_ = resolve;
        });
      }
    };

    // Create main frame sensor.
    await test_driver.set_permission({name: permissionName}, 'granted');
    await test_driver.create_virtual_sensor(testDriverName);
    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);

    // Create cross-origin iframe and a sensor inside it.
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src =
        'https://{{domains[www1]}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';
    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    t.add_cleanup(async () => {
      await send_message_to_iframe(iframe, {command: 'stop_sensor'});
      iframe.parentNode.removeChild(iframe);
    });
    await iframeLoadWatcher.wait_for('load');
    const iframeSensorWatcher = new IframeSensorReadingEventWatcher(t);
    await send_message_to_iframe(
        iframe, {command: 'create_sensor', sensorData});

    // Start the test by focusing the main frame. It is already focused by
    // default, but this makes the test easier to follow.
    // When the main frame is focused, it sensor is expected to fire "reading"
    // events and provide access to new reading values while the sensor in the
    // cross-origin iframe is not.
    window.focus();

    // Start both sensors. They should both have the same state: active, but no
    // readings have been provided to them yet.
    await send_message_to_iframe(iframe, {command: 'start_sensor'});
    sensor.start();
    await sensorWatcher.wait_for('activate');
    assert_false(
        await send_message_to_iframe(iframe, {command: 'has_reading'}));
    assert_false(sensor.hasReading);

    // We store `reading` here because we want to make sure the very same
    // value is accepted later.
    const reading = readings.next().value;
    await Promise.all([
      sensorWatcher.wait_for('reading'),
      test_driver.update_virtual_sensor(testDriverName, reading),
      // Since we do not wait for the iframe sensor's "reading" event, it could
      // arguably be delivered later. There are enough async calls happening
      // that IframeSensorReadingEventWatcher would end up catching it and
      // throwing an error.
    ]);
    assert_true(sensor.hasReading);
    assert_false(
        await send_message_to_iframe(iframe, {command: 'has_reading'}));

    // Save sensor data for later before the sensor is stopped.
    const savedMainFrameSensorReadings = serialize_sensor_data(sensor);

    sensor.stop();
    await send_message_to_iframe(iframe, {command: 'stop_sensor'});

    // Now focus the cross-origin iframe. The situation should be the opposite:
    // the sensor in the main frame should not fire any "reading" events or
    // provide access to updated readings, but the sensor in the iframe should.
    iframe.contentWindow.focus();

    // Start both sensors. They should both have the same state: active, but no
    // readings have been provided to them yet.
    await send_message_to_iframe(iframe, {command: 'start_sensor'});
    sensor.start();
    await sensorWatcher.wait_for('activate');
    assert_false(
        await send_message_to_iframe(iframe, {command: 'has_reading'}));
    assert_false(sensor.hasReading);

    const [serializedIframeSensor] = await Promise.all([
      iframeSensorWatcher.wait_for_reading(),
      test_driver.update_virtual_sensor(testDriverName, reading),
    ]);
    assert_true(await send_message_to_iframe(iframe, {command: 'has_reading'}));
    assert_false(sensor.hasReading);

    assert_sensor_reading_is_null(sensor);

    assert_sensor_reading_equals(
        savedMainFrameSensorReadings, serializedIframeSensor,
        {ignoreTimestamps: true});

    // We could check that serializedIframeSensor.timestamp (adjusted to this
    // frame by adding the iframe's timeOrigin and substracting
    // performance.timeOrigin) is greater than
    // savedMainFrameSensorReadings.timestamp (or other timestamps prior to the
    // last test_driver.update_virtual_sensor() call), but this is surprisingly
    // tricky and flaky due to the fact that we are using timestamps from
    // cross-origin frames.
    //
    // On Chrome on Windows (M120 at the time of writing), for example, the
    // difference between timeOrigin values is sometimes off by more than 10ms
    // from the real difference, and allowing for this much jitter makes the
    // test not test something meaningful.
  }, `${sensorName}: unfocused sensors in cross-origin frames are not updated`);

  sensor_test(async (t, readings) => {
    // Create main frame sensor.
    await test_driver.set_permission({name: permissionName}, 'granted');
    await test_driver.create_virtual_sensor(testDriverName);
    const sensor = new sensorType();
    t.add_cleanup(async () => {
      sensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher =
        new EventWatcher(t, sensor, ['activate', 'reading', 'error']);

    // Create same-origin iframe and a sensor inside it.
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src = 'https://{{host}}:{{ports[https][0]}}/resources/blank.html';
    // Create sensor inside same-origin nested browsing context.
    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    t.add_cleanup(() => {
      if (iframeSensor) {
        iframeSensor.stop();
      }
      iframe.parentNode.removeChild(iframe);
    });
    await iframeLoadWatcher.wait_for('load');
    // We deliberately create the sensor here instead of using
    // send_messge_to_iframe() because this is a same-origin iframe, and we can
    // therefore use EventWatcher() to wait for "reading" events a lot more
    // easily.
    const iframeSensor = new iframe.contentWindow[sensorName]();
    const iframeSensorWatcher =
        new EventWatcher(t, iframeSensor, ['activate', 'error', 'reading']);

    // Focus a different same-origin window each time and check that everything
    // works the same.
    for (const windowObject of [window, iframe.contentWindow]) {
      windowObject.focus();

      iframeSensor.start();
      sensor.start();
      await Promise.all([
        iframeSensorWatcher.wait_for('activate'),
        sensorWatcher.wait_for('activate')
      ]);

      assert_false(sensor.hasReading);
      assert_false(iframeSensor.hasReading);

      // We store `reading` here because we want to make sure the very same
      // value is accepted later.
      const reading = readings.next().value;
      await Promise.all([
        test_driver.update_virtual_sensor(testDriverName, reading),
        iframeSensorWatcher.wait_for('reading'),
        sensorWatcher.wait_for('reading')
      ]);

      assert_greater_than(
          iframe.contentWindow.performance.timeOrigin, performance.timeOrigin,
          'iframe\'s time origin must be higher than the main window\'s');

      // Check that the timestamps are similar enough to indicate that this is
      // the same reading that was delivered to both sensors.
      // The values are not identical due to how high resolution time is
      // coarsened.
      const translatedIframeSensorTimestamp = iframeSensor.timestamp +
          iframe.contentWindow.performance.timeOrigin - performance.timeOrigin;
      assert_approx_equals(
          translatedIframeSensorTimestamp, sensor.timestamp,
          ALLOWED_JITTER_IN_MS);

      // Do not compare timestamps here because of the reasons above.
      assert_sensor_reading_equals(
          sensor, iframeSensor, {ignoreTimestamps: true});

      // Stop all sensors so we can use the same value in `reading` on every
      // loop iteration.
      iframeSensor.stop();
      sensor.stop();
    }
  }, `${sensorName}: sensors in same-origin frames are updated if one of the frames is focused`);

  promise_test(async t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src =
        'https://{{host}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';

    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeLoadWatcher.wait_for('load');

    // Create sensor in the iframe.
    await test_driver.set_permission({name: permissionName}, 'granted');
    await test_driver.create_virtual_sensor(testDriverName);
    iframe.contentWindow.focus();
    const iframeSensor = new iframe.contentWindow[sensorName]();
    t.add_cleanup(async () => {
      iframeSensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    const sensorWatcher = new EventWatcher(t, iframeSensor, ['activate']);
    iframeSensor.start();
    await sensorWatcher.wait_for('activate');

    // Remove iframe from main document and change focus. When focus changes,
    // we need to determine whether a sensor must have its execution suspended
    // or resumed (section 4.2.3, "Focused Area" of the Generic Sensor API
    // spec). In Blink, this involves querying a frame, which might no longer
    // exist at the time of the check.
    iframe.parentNode.removeChild(iframe);
    window.focus();
  }, `${sensorName}: losing a document's frame with an active sensor does not crash`);

  promise_test(async t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src =
        'https://{{host}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';

    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeLoadWatcher.wait_for('load');

    // Create sensor in the iframe.
    await test_driver.set_permission({name: permissionName}, 'granted');
    await test_driver.create_virtual_sensor(testDriverName);
    const iframeSensor = new iframe.contentWindow[sensorName]();
    t.add_cleanup(async () => {
      iframeSensor.stop();
      await test_driver.remove_virtual_sensor(testDriverName);
    });
    assert_not_equals(iframeSensor, null);

    // Remove iframe from main document. |iframeSensor| no longer has a
    // non-null browsing context. Calling start() should probably throw an
    // error when called from a non-fully active document, but that depends on
    // https://github.com/w3c/sensors/issues/415
    iframe.parentNode.removeChild(iframe);
    iframeSensor.start();
  }, `${sensorName}: calling start() in a non-fully active document does not crash`);
}
