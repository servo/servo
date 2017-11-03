let unreached = event => {
  assert_unreached(event.error.name + ": " + event.error.message);
};

let properties = {
  'AmbientLightSensor' : ['timestamp', 'illuminance'],
  'Accelerometer' : ['timestamp', 'x', 'y', 'z'],
  'LinearAccelerationSensor' : ['timestamp', 'x', 'y', 'z'],
  'Gyroscope' : ['timestamp', 'x', 'y', 'z'],
  'Magnetometer' : ['timestamp', 'x', 'y', 'z'],
  'AbsoluteOrientationSensor' : ['timestamp', 'quaternion'],
  'RelativeOrientationSensor' : ['timestamp', 'quaternion']
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
    sensor.onreading = t.step_func_done(() => {
      assert_reading_not_null(sensor);
      assert_true(sensor.hasReading);
      sensor.stop();
      assert_reading_null(sensor);
      assert_false(sensor.hasReading);
    });
    sensor.onerror = t.step_func_done(unreached);
    sensor.start();
  }, `${sensorType.name}: Test that 'onreading' is called and sensor reading is valid`);

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
  }, `${sensorType.name}: sensor reading is correct`);

  async_test(t => {
    let sensor = new sensorType();
    let cachedTimeStamp1;
    sensor.onactivate = () => {
      cachedTimeStamp1 = sensor.timestamp;
    };
    sensor.onerror = t.step_func_done(unreached);
    sensor.start();
    t.step_timeout(() => {
      sensor.onreading = t.step_func_done(() => {
        //sensor.timestamp changes.
        let cachedTimeStamp2 = sensor.timestamp;
        assert_greater_than(cachedTimeStamp2, cachedTimeStamp1);
        sensor.stop();
      });
    }, 1000);
  }, `${sensorType.name}: sensor timestamp is updated when time passes`);

  async_test(t => {
    let sensor = new sensorType();
    sensor.onerror = t.step_func_done(unreached);
    assert_false(sensor.activated);
    sensor.onreading = t.step_func_done(() => {
      assert_true(sensor.activated);
      sensor.stop();
      assert_false(sensor.activated);
    });
    sensor.start();
    assert_false(sensor.activated);
  }, `${sensorType.name}: Test that sensor can be successfully created and its states are correct.`);

  test(() => {
    let sensor, start_return;
    sensor = new sensorType();
    sensor.onerror = unreached;
    start_return = sensor.start();
    assert_equals(start_return, undefined);
    sensor.stop();
  }, `${sensorType.name}: sensor.start() returns undefined`);

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
  }, `${sensorType.name}: no exception is thrown when calling start() on already started sensor`);

  test(() => {
    let sensor, stop_return;
    sensor = new sensorType();
    sensor.onerror = unreached;
    sensor.start();
    stop_return = sensor.stop();
    assert_equals(stop_return, undefined);
  }, `${sensorType.name}: sensor.stop() returns undefined`);

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
  }, `${sensorType.name}: no exception is thrown when calling stop() on already stopped sensor`);

  promise_test(() => {
    return new Promise((resolve,reject) => {
      let iframe = document.createElement('iframe');
      iframe.srcdoc = '<script>' +
                      '  window.onmessage = message => {' +
                      '    if (message.data === "LOADED") {' +
                      '      try {' +
                      '        new ' + sensorType.name + '();' +
                      '        parent.postMessage("FAIL", "*");' +
                      '      } catch (e) {' +
                      '        parent.postMessage(e.name, "*");' +
                      '      }' +
                      '    }' +
                      '   };' +
                      '<\/script>';
      iframe.onload = () => iframe.contentWindow.postMessage('LOADED', '*');
      document.body.appendChild(iframe);
      window.onmessage = message => {
        if (message.data == 'SecurityError') {
          resolve();
        } else {
          reject();
        }
      }
    });
  }, `${sensorType.name}: throw a 'SecurityError' when constructing sensor object within iframe`);

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
  }, `${sensorType.name}: sensor readings can not be fired on the background tab`);
}

function runGenericSensorInsecureContext(sensorType) {
  test(() => {
    assert_false(sensorType in window, `${sensorType} must not be exposed`);
  }, `${sensorType} is not exposed in an insecure context`);
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
  }, `${sensorType.name}: 'onerror' event is fired when sensor is not supported`);
}
