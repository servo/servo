'use strict';

async function checkQuaternion(
    t, sensorType, testDriverName, permissionName, readings) {
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
  assert_equals(sensor.quaternion.length, 4, 'Quaternion length must be 4');
  assert_true(
      sensor.quaternion instanceof Array, 'Quaternion is must be array');
};

async function checkPopulateMatrix(
    t, sensorProvider, sensorType, testDriverName, permissionName, readings) {
  await test_driver.set_permission({name: permissionName}, 'granted');
  await test_driver.create_virtual_sensor(testDriverName);
  const sensor = new sensorType();
  t.add_cleanup(async () => {
    sensor.stop();
    await test_driver.remove_virtual_sensor(testDriverName);
  });
  const sensorWatcher =
      new EventWatcher(t, sensor, ['activate', 'reading', 'error']);

  // Throws with insufficient buffer space.
  assert_throws_js(
      TypeError, () => sensor.populateMatrix(new Float32Array(15)));

  // Throws if no orientation data available.
  assert_throws_dom(
      'NotReadableError', () => sensor.populateMatrix(new Float32Array(16)));

  // Throws if passed SharedArrayBuffer view.
  assert_throws_js(
      TypeError,
      // See https://github.com/whatwg/html/issues/5380 for why not `new
      // SharedArrayBuffer()` WebAssembly.Memory's size is in multiples of 64KiB
      () => sensor.populateMatrix(new Float32Array(
          new WebAssembly.Memory({shared: true, initial: 1, maximum: 1})
              .buffer)));

  sensor.start();
  await sensorWatcher.wait_for('activate');

  await Promise.all([
    test_driver.update_virtual_sensor(testDriverName, readings.next().value),
    sensorWatcher.wait_for('reading')
  ]);

  // Works for all supported types.
  const rotationMatrix32 = new Float32Array(16);
  sensor.populateMatrix(rotationMatrix32);
  assert_array_approx_equals(rotationMatrix32, kRotationMatrix, kEpsilon);

  let rotationMatrix64 = new Float64Array(16);
  sensor.populateMatrix(rotationMatrix64);
  assert_array_approx_equals(rotationMatrix64, kRotationMatrix, kEpsilon);

  let rotationDOMMatrix = new DOMMatrix();
  sensor.populateMatrix(rotationDOMMatrix);
  assert_array_approx_equals(
      rotationDOMMatrix.toFloat64Array(), kRotationMatrix, kEpsilon);

  // Sets every matrix element.
  rotationMatrix64.fill(123);
  sensor.populateMatrix(rotationMatrix64);
  assert_array_approx_equals(rotationMatrix64, kRotationMatrix, kEpsilon);
}

function runOrientationSensorTests(sensorData, readingData) {
  validate_sensor_data(sensorData);
  validate_reading_data(readingData);

  const {sensorName, permissionName, testDriverName} = sensorData;
  const sensorType = self[sensorName];

  const readings = new RingBuffer(readingData.readings);

  promise_test(async t => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    return checkQuaternion(
        t, sensorType, testDriverName, permissionName, readings);
  }, `${sensorName}.quaternion return a four-element FrozenArray.`);

  promise_test(async (t, sensorProvider) => {
    assert_implements(sensorName in self, `${sensorName} is not supported.`);
    return checkPopulateMatrix(
        t, sensorProvider, sensorType, testDriverName, permissionName,
        readings);
  }, `${sensorName}.populateMatrix() method works correctly.`);
}
