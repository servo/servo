async function send_message_to_iframe(iframe, message, reply) {
  if (reply === undefined) {
    reply = 'success';
  }

  return new Promise((resolve, reject) => {
    window.addEventListener('message', (e) => {
      if (e.data.command !== message.command) {
        reject(`Expected reply with command '${message.command}', got '${e.data.command}' instead`);
        return;
      }
      if (e.data.result === reply) {
        resolve();
      } else {
        reject(`Got unexpected reply '${e.data.result}' to command '${message.command}', expected '${reply}'`);
      }
    }, { once: true });
    iframe.contentWindow.postMessage(message, '*');
  });
}

function run_generic_sensor_iframe_tests(sensorName) {
  const sensorType = self[sensorName];
  const featurePolicies = get_feature_policies_for_sensor(sensorName);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src = 'https://{{domains[www1]}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';

    // Create sensor inside cross-origin nested browsing context.
    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeLoadWatcher.wait_for('load');
    await send_message_to_iframe(iframe, {command: 'create_sensor',
                                          type: sensorName});

    // Focus on the main frame and test that sensor receives readings.
    window.focus();
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ['reading', 'error']);
    sensor.start();

    await sensorWatcher.wait_for('reading');
    const cachedTimeStamp = sensor.timestamp;

    // Focus on the cross-origin frame and verify that sensor reading updates in
    // the top level browsing context are suspended.
    iframe.contentWindow.focus();
    await send_message_to_iframe(iframe, {command: 'start_sensor'});
    assert_equals(cachedTimeStamp, sensor.timestamp);

    // Focus on the main frame, verify that sensor reading updates are resumed.
    window.focus();
    await sensorWatcher.wait_for('reading');
    assert_greater_than(sensor.timestamp, cachedTimeStamp);
    sensor.stop();

    // Verify that sensor in cross-origin frame is suspended.
    await send_message_to_iframe(iframe, {command: 'is_sensor_suspended'}, true);
    await send_message_to_iframe(iframe, {command: 'reset_sensor_backend'});

    // Remove iframe from main document.
    iframe.parentNode.removeChild(iframe);
  }, `${sensorName}: sensor is suspended and resumed when focus traverses from\
 to cross-origin frame`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src = 'https://{{host}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';

    // Create sensor inside same-origin nested browsing context.
    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeLoadWatcher.wait_for('load');
    await send_message_to_iframe(iframe, {command: 'create_sensor',
                                          type: sensorName});

    // Focus on main frame and test that sensor receives readings.
    window.focus();
    const sensor = new sensorType({
      // generic_sensor_mocks.js uses a default frequency of 5Hz for sensors.
      // We deliberately use a higher frequency here to make it easier to spot
      // spurious, unexpected 'reading' events caused by the main frame's
      // sensor not stopping early enough.
      // TODO(rakuco): Create a constant with the 5Hz default frequency instead
      // of using magic numbers.
      frequency: 15
    });
    const sensorWatcher = new EventWatcher(t, sensor, ['reading', 'error']);
    sensor.start();
    await sensorWatcher.wait_for('reading');
    let cachedTimeStamp = sensor.timestamp;

    // Stop sensor in main frame, so that sensorWatcher would not receive
    // readings while sensor in iframe is started. Sensors that are active and
    // belong to the same-origin context are not suspended automatically when
    // focus changes to another same-origin iframe, so if we do not explicitly
    // stop them we may receive extra 'reading' events that cause the test to
    // fail (see e.g. https://crbug.com/857520).
    sensor.stop();

    iframe.contentWindow.focus();
    await send_message_to_iframe(iframe, {command: 'start_sensor'});

    // Start sensor on main frame, verify that readings are updated.
    window.focus();
    sensor.start();
    await sensorWatcher.wait_for('reading');
    assert_greater_than(sensor.timestamp, cachedTimeStamp);
    cachedTimeStamp = sensor.timestamp;
    sensor.stop();

    // Verify that sensor in nested browsing context is not suspended.
    await send_message_to_iframe(iframe, {command: 'is_sensor_suspended'}, false);

    // Verify that sensor in top level browsing context is receiving readings.
    iframe.contentWindow.focus();
    sensor.start();
    await sensorWatcher.wait_for('reading');
    assert_greater_than(sensor.timestamp, cachedTimeStamp);
    sensor.stop();
    await send_message_to_iframe(iframe, {command: 'reset_sensor_backend'});

    // Remove iframe from main document.
    iframe.parentNode.removeChild(iframe);
  }, `${sensorName}: sensor is not suspended when focus traverses from\
 to same-origin frame`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    const iframe = document.createElement('iframe');
    iframe.allow = featurePolicies.join(';') + ';';
    iframe.src = 'https://{{host}}:{{ports[https][0]}}/generic-sensor/resources/iframe_sensor_handler.html';

    // Create sensor in the iframe (we do not care whether this is a
    // cross-origin nested context in this test).
    const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
    document.body.appendChild(iframe);
    await iframeLoadWatcher.wait_for('load');
    await send_message_to_iframe(iframe, {command: 'create_sensor',
                                          type: sensorName});
    iframe.contentWindow.focus();
    await send_message_to_iframe(iframe, {command: 'start_sensor'});

    // Remove iframe from main document and change focus. When focus changes,
    // we need to determine whether a sensor must have its execution suspended
    // or resumed (section 4.2.3, "Focused Area" of the Generic Sensor API
    // spec). In Blink, this involves querying a frame, which might no longer
    // exist at the time of the check.
    // Note that we cannot send the "reset_sensor_backend" command because the
    // iframe is discarded with the removeChild call.
    iframe.parentNode.removeChild(iframe);
    window.focus();
  }, `${sensorName}: losing a document's frame with an active sensor does not crash`);
}
