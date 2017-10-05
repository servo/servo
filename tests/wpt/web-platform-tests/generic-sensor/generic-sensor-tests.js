let unreached = event => {
    assert_unreached(event.error.name + ":" + event.error.message);
};

let properties = {
    'AmbientLightSensor' : ['timestamp', 'illuminance'],
    'Accelerometer' : ['timestamp', 'x', 'y', 'z'],
    'Gyroscope' : ['timestamp', 'x', 'y', 'z'],
    'Magnetometer' : ['timestamp', 'x', 'y', 'z']
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
  let arr = new Array();
  for (let property in properties[sensor.constructor.name]) {
    let propertyName = properties[sensor.constructor.name][property];
    arr[property] = sensor[propertyName];
  }
  return arr;
}

function runGenericSensorTests(sensorType) {
  async_test(t => {
    let sensor = new sensorType();
    sensor.onchange = t.step_func_done(() => {
      assert_reading_not_null(sensor);
      sensor.stop();
      assert_reading_null(sensor);
    });
    sensor.onerror = t.step_func_done(unreached);
    sensor.start();
  }, "Test that 'onchange' is called and sensor reading is valid");

  async_test(t => {
    let sensor1 = new sensorType();
    let sensor2 = new sensorType();
    sensor1.onactivate = t.step_func_done(() => {
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
    });
    sensor1.onerror = t.step_func_done(unreached);
    sensor2.onerror = t.step_func_done(unreached);
    sensor1.start();
    sensor2.start();
  }, "sensor reading is correct");

  async_test(t => {
    let sensor = new sensorType();
    let cachedTimeStamp1;
    sensor.onactivate = () => {
      cachedTimeStamp1 = sensor.timestamp;
    };
    sensor.onerror = t.step_func_done(unreached);
    sensor.start();
    t.step_timeout(() => {
      sensor.onchange = t.step_func_done(() => {
        //sensor.timestamp changes.
        let cachedTimeStamp2 = sensor.timestamp;
        assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
        sensor.stop();
      });
    }, 1000);
  }, "sensor timestamp is updated when time passes");

  async_test(t => {
    let sensor = new sensorType();
    sensor.onerror = t.step_func_done(unreached);
    assert_false(sensor.activated);
    sensor.onchange = t.step_func_done(() => {
      assert_true(sensor.activated);
      sensor.stop();
      assert_false(sensor.activated);
    });
    sensor.start();
    assert_false(sensor.activated);
  }, "Test that sensor can be successfully created and its states are correct.");

  test(() => {
    let sensor, start_return;
    sensor = new sensorType();
    sensor.onerror = unreached;
    start_return = sensor.start();
    assert_equals(start_return, undefined);
    sensor.stop();
  }, "sensor.start() returns undefined");

  test(() => {
    try {
      let sensor = new sensorType();
      sensor.onerror = unreached;
      sensor.start();
      sensor.start();
      assert_false(sensor.activated);
      sensor.stop();
    } catch (e) {
       assert_unreached(e.name + ": " + e.message);
    }
  }, "no exception is thrown when calling start() on already started sensor");

  test(() => {
    let sensor, stop_return;
    sensor = new sensorType();
    sensor.onerror = unreached;
    sensor.start();
    stop_return = sensor.stop();
    assert_equals(stop_return, undefined);
  }, "sensor.stop() returns undefined");

  test(() => {
    try {
      let sensor = new sensorType();
      sensor.onerror = unreached;
      sensor.start();
      sensor.stop();
      sensor.stop();
      assert_false(sensor.activated);
    } catch (e) {
       assert_unreached(e.name + ": " + e.message);
    }
  }, "no exception is thrown when calling stop() on already stopped sensor");

  async_test(t => {
    window.onmessage = t.step_func(e => {
      assert_equals(e.data, "SecurityError");
      t.done();
    });
  }, "throw a 'SecurityError' when firing sensor readings within iframes");

  async_test(t => {
    let sensor = new sensorType();
    sensor.onactivate = t.step_func(() => {
      assert_reading_not_null(sensor);
      let cachedSensor1 = reading_to_array(sensor);
      let win = window.open('', '_blank');
      t.step_timeout(() => {
        let cachedSensor2 = reading_to_array(sensor);
        win.close();
        sensor.stop();
        assert_array_equals(cachedSensor1, cachedSensor2);
        t.done();
      }, 1000);
    });
    sensor.onerror = t.step_func_done(unreached);
    sensor.start();
  }, "sensor readings can not be fired on the background tab");
}

function runGenericSensorInsecureContext(sensorType) {
  test(() => {
    assert_throws('SecurityError', () => { new sensorType(); });
  }, "throw a 'SecurityError' when construct sensor in an insecure context");
}

function runGenericSensorOnerror(sensorType) {
  async_test(t => {
    let sensor = new sensorType();
    sensor.onactivate = t.step_func_done(assert_unreached);
    sensor.onerror = t.step_func_done(event => {
      assert_false(sensor.activated);
      assert_equals(event.error.name, 'NotReadableError');
    });
    sensor.start();
  }, "'onerror' event is fired when sensor is not supported");
}
