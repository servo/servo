'use strict';

const kDefaultReading = [
    [ 1, 0, 0, 0 ]  // 180 degrees around X axis.
];
const kRotationMatrix = [1,  0,  0,  0,
                         0, -1,  0,  0,
                         0,  0, -1,  0,
                         0,  0,  0,  1];
const kReadings = {
    readings: kDefaultReading,
    expectedReadings: kDefaultReading,
    expectedRemappedReadings: [
        // For 'orientation.angle == 270', which is set for tests at
        // at SensorProxy::GetScreenOrientationAngle().
        [-0.707107, 0.707107, 0, 0]
    ]
};

async function checkQuaternion(t, sensorType) {
  const sensor = new sensorType();
  const eventWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
  sensor.start();

  await eventWatcher.wait_for("reading");
  assert_equals(sensor.quaternion.length, 4);
  assert_true(sensor.quaternion instanceof Array);
  sensor.stop();
};

async function checkPopulateMatrix(t, sensorProvider, sensorType) {
  const sensor = new sensorType();
  const eventWatcher = new EventWatcher(t, sensor, ["reading", "error"]);

  // Throws with insufficient buffer space.
  assert_throws_js(TypeError,
      () => sensor.populateMatrix(new Float32Array(15)));

  // Throws if no orientation data available.
  assert_throws_dom('NotReadableError',
      () => sensor.populateMatrix(new Float32Array(16)));

  // Throws if passed SharedArrayBuffer view.
  assert_throws_js(TypeError,
      // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
      // WebAssembly.Memory's size is in multiples of 64 KiB
      () => sensor.populateMatrix(new Float32Array(new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer)));

  sensor.start();

  const mockSensor = await sensorProvider.getCreatedSensor(sensorType.name);
  await mockSensor.setSensorReading(kDefaultReading);

  await eventWatcher.wait_for("reading");

  // Works for all supported types.
  const rotationMatrix32 = new Float32Array(16);
  sensor.populateMatrix(rotationMatrix32);
  assert_array_equals(rotationMatrix32, kRotationMatrix);

  let rotationMatrix64 = new Float64Array(16);
  sensor.populateMatrix(rotationMatrix64);
  assert_array_equals(rotationMatrix64, kRotationMatrix);

  let rotationDOMMatrix = new DOMMatrix();
  sensor.populateMatrix(rotationDOMMatrix);
  assert_array_equals(rotationDOMMatrix.toFloat64Array(), kRotationMatrix);

  // Sets every matrix element.
  rotationMatrix64.fill(123);
  sensor.populateMatrix(rotationMatrix64);
  assert_array_equals(rotationMatrix64, kRotationMatrix);

  sensor.stop();
}

function runOrienationSensorTests(sensorName) {
  const sensorType = self[sensorName];

  sensor_test(async t => {
    assert_true(sensorName in self);
    return checkQuaternion(t, sensorType);
  }, `${sensorName}.quaternion return a four-element FrozenArray.`);

  sensor_test(async (t, sensorProvider) => {
    assert_true(sensorName in self);
    return checkPopulateMatrix(t, sensorProvider, sensorType);
  }, `${sensorName}.populateMatrix() method works correctly.`);
}
