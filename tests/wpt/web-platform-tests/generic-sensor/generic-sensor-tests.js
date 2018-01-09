const properties = {
  'AmbientLightSensor' : ['timestamp', 'illuminance'],
  'Accelerometer' : ['timestamp', 'x', 'y', 'z'],
  'LinearAccelerationSensor' : ['timestamp', 'x', 'y', 'z'],
  "GravitySensor" : ['timestamp', 'x', 'y', 'z'],
  'Gyroscope' : ['timestamp', 'x', 'y', 'z'],
  'Magnetometer' : ['timestamp', 'x', 'y', 'z'],
  "UncalibratedMagnetometer" : ['timestamp', 'x', 'y', 'z',
                                'xBias', 'yBias', 'zBias'],
  'AbsoluteOrientationSensor' : ['timestamp', 'quaternion'],
  'RelativeOrientationSensor' : ['timestamp', 'quaternion'],
  'GeolocationSensor' : ['timestamp', 'latitude', 'longitude', 'altitude',
                         'accuracy', 'altitudeAccuracy', 'heading', 'speed']
};

function assert_reading_not_null(sensor) {
  for (let property in properties[sensor.constructor.name]) {
    let propertyName = properties[sensor.constructor.name][property];
    assert_not_equals(sensor[propertyName], null);
  }
}

function assert_reading_null(sensor) {
  for (let property in properties[sensor.constructor.name]) {
    let propertyName = properties[sensor.constructor.name][property];
    assert_equals(sensor[propertyName], null);
  }
}

function reading_to_array(sensor) {
  const arr = new Array();
  for (let property in properties[sensor.constructor.name]) {
    let propertyName = properties[sensor.constructor.name][property];
    arr[property] = sensor[propertyName];
  }
  return arr;
}

function runGenericSensorTests(sensorType) {
  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("reading");
    assert_reading_not_null(sensor);
    assert_true(sensor.hasReading);

    sensor.stop();
    assert_reading_null(sensor);
    assert_false(sensor.hasReading);
  }, `${sensorType.name}: Test that 'onreading' is called and sensor reading is valid`);

  promise_test(async t => {
    const sensor1 = new sensorType();
    const sensor2 = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor1, ["reading", "error"]);
    sensor2.start();
    sensor1.start();

    await sensorWatcher.wait_for("reading");
    // Reading values are correct for both sensors.
    assert_reading_not_null(sensor1);
    assert_reading_not_null(sensor2);

    //After first sensor stops its reading values are null,
    //reading values for the second sensor remains
    sensor1.stop();
    assert_reading_null(sensor1);
    assert_reading_not_null(sensor2);
    sensor2.stop();
    assert_reading_null(sensor2);
  }, `${sensorType.name}: sensor reading is correct`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp1 = sensor.timestamp;

    await sensorWatcher.wait_for("reading");
    const cachedTimeStamp2 = sensor.timestamp;

    assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
    sensor.stop();
  }, `${sensorType.name}: sensor timestamp is updated when time passes`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    assert_false(sensor.activated);
    sensor.start();
    assert_false(sensor.activated);

    await sensorWatcher.wait_for("activate");
    assert_true(sensor.activated);

    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorType.name}: Test that sensor can be successfully created and its states are correct.`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    const start_return = sensor.start();

    await sensorWatcher.wait_for("activate");
    assert_equals(start_return, undefined);
    sensor.stop();
  }, `${sensorType.name}: sensor.start() returns undefined`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();
    sensor.start();

    await sensorWatcher.wait_for("activate");
    assert_true(sensor.activated);
    sensor.stop();
  }, `${sensorType.name}: no exception is thrown when calling start() on already started sensor`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("activate");
    const stop_return = sensor.stop();
    assert_equals(stop_return, undefined);
  }, `${sensorType.name}: sensor.stop() returns undefined`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["activate", "error"]);
    sensor.start();

    await sensorWatcher.wait_for("activate");
    sensor.stop();
    sensor.stop();
    assert_false(sensor.activated);
  }, `${sensorType.name}: no exception is thrown when calling stop() on already stopped sensor`);

  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
    const visibilityChangeWatcher = new EventWatcher(t, document, "visibilitychange");
    sensor.start();

    await sensorWatcher.wait_for("reading");
    assert_reading_not_null(sensor);
    const cachedSensor1 = reading_to_array(sensor);

    const win = window.open('', '_blank');
    await visibilityChangeWatcher.wait_for("visibilitychange");
    const cachedSensor2 = reading_to_array(sensor);

    win.close();
    sensor.stop();
    assert_array_equals(cachedSensor1, cachedSensor2);
  }, `${sensorType.name}: sensor readings can not be fired on the background tab`);
}

function runGenericSensorInsecureContext(sensorType) {
  test(() => {
    assert_false(sensorType in window, `${sensorType} must not be exposed`);
  }, `${sensorType} is not exposed in an insecure context`);
}

function runGenericSensorOnerror(sensorType) {
  promise_test(async t => {
    const sensor = new sensorType();
    const sensorWatcher = new EventWatcher(t, sensor, ["error", "activate"]);
    sensor.start();

    const event = await sensorWatcher.wait_for("error");
    assert_false(sensor.activated);
    assert_equals(event.error.name, 'NotReadableError');
  }, `${sensorType.name}: 'onerror' event is fired when sensor is not supported`);
}
