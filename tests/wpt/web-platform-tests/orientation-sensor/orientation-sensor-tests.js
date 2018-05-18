//IEEE 754: single precision retricts to 7 decimal digits
const float_precision = 1e-7;

function create_matrix(quat) {
  const X = quat[0];
  const Y = quat[1];
  const Z = quat[2];
  const W = quat[3];
  const mat = new Array(
    1-2*Y*Y-2*Z*Z, 2*X*Y-2*Z*W, 2*X*Z+2*Y*W, 0,
    2*X*Y+2*Z*W, 1-2*X*X-2*Z*Z, 2*Y*Z-2*X*W, 0,
    2*X*Z-2*Y*W, 2*Y*Z+2*W*X, 1-2*X*X-2*Y*Y, 0,
    0, 0, 0, 1
  );
  return mat;
}

async function checkQuaternion(t, sensorType) {
  const sensor = new sensorType();
  const eventWatcher = new EventWatcher(t, sensor, ["reading", "error"]);
  sensor.start();

  await eventWatcher.wait_for("reading");
  assert_equals(sensor.quaternion.length, 4);
  assert_true(sensor.quaternion instanceof Array);
  sensor.stop();
};

async function checkPopulateMatrix(t, sensorType) {
  const sensor = new sensorType();
  const eventWatcher = new EventWatcher(t, sensor, ["reading", "error"]);

  //Throws with insufficient buffer space.
  assert_throws({ name: 'TypeError' }, () => sensor.populateMatrix(new Float32Array(15)));

  //Throws if no orientation data available.
  assert_throws({ name: 'NotReadableError' }, () => sensor.populateMatrix(new Float32Array(16)));

  if (window.SharedArrayBuffer) {
    // Throws if passed SharedArrayBuffer view.
    assert_throws({ name: 'TypeError' }, () => sensor.populateMatrix(new Float32Array(new SharedArrayBuffer(16))));
  }

  sensor.start();
  await eventWatcher.wait_for("reading");
  const quat = sensor.quaternion;
  const mat_expect = create_matrix(quat);

  // Works for all supported types.
  const mat_32 = new Float32Array(16);
  sensor.populateMatrix(mat_32);
  assert_array_approx_equals(mat_32, mat_expect, float_precision);

  const mat_64 = new Float64Array(16);
  sensor.populateMatrix(mat_64);
  assert_array_equals(mat_64, mat_expect);

  const mat_dom = new DOMMatrix();
  sensor.populateMatrix(mat_dom);
  assert_array_equals(mat_dom.toFloat64Array(), mat_expect);

  // Sets every matrix element.
  mat_64.fill(123);
  sensor.populateMatrix(mat_64);
  assert_array_equals(mat_64, mat_expect);

  sensor.stop();
}

function runOrienationSensorTests(sensorName) {
  const sensorType = self[sensorName];

  sensor_test(async t => {
    assert_true(sensorName in self);
    return checkQuaternion(t, sensorType);
  }, `${sensorName}.quaternion return a four-element FrozenArray.`);

  sensor_test(async t => {
    assert_true(sensorName in self);
    return checkPopulateMatrix(t, sensorType);
  }, `${sensorName}.populateMatrix() method works correctly.`);
}

