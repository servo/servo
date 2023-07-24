// META: global=window,dedicatedworker
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

var defaultInit =
    {
      timestamp: 1234,
      channels: 2,
      sampleRate: 8000,
      frames: 100,
    }

function
createDefaultAudioData() {
  return make_audio_data(
      defaultInit.timestamp, defaultInit.channels, defaultInit.sampleRate,
      defaultInit.frames);
}

test(t => {
  let local_data = new Float32Array(defaultInit.channels * defaultInit.frames);

  let audio_data_init = {
    timestamp: defaultInit.timestamp,
    data: local_data,
    numberOfFrames: defaultInit.frames,
    numberOfChannels: defaultInit.channels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32-planar',
  }

  let data = new AudioData(audio_data_init);

  assert_equals(data.timestamp, defaultInit.timestamp, 'timestamp');
  assert_equals(data.numberOfFrames, defaultInit.frames, 'frames');
  assert_equals(data.numberOfChannels, defaultInit.channels, 'channels');
  assert_equals(data.sampleRate, defaultInit.sampleRate, 'sampleRate');
  assert_equals(
      data.duration, defaultInit.frames / defaultInit.sampleRate * 1_000_000,
      'duration');
  assert_equals(data.format, 'f32-planar', 'format');

  // Create an Int16 array of the right length.
  let small_data = new Int16Array(defaultInit.channels * defaultInit.frames);

  let wrong_format_init = {...audio_data_init};
  wrong_format_init.data = small_data;

  // Creating `f32-planar` AudioData from Int16 from should throw.
  assert_throws_js(TypeError, () => {
    let data = new AudioData(wrong_format_init);
  }, `AudioDataInit.data needs to be big enough`);

  var members = [
    'timestamp',
    'data',
    'numberOfFrames',
    'numberOfChannels',
    'sampleRate',
    'format',
  ];

  for (const member of members) {
    let incomplete_init = {...audio_data_init};
    delete incomplete_init[member];

    assert_throws_js(
        TypeError, () => {let data = new AudioData(incomplete_init)},
        'AudioData requires \'' + member + '\'');
  }

  let invalid_init = {...audio_data_init};
  invalid_init.numberOfFrames = 0

  assert_throws_js(
      TypeError, () => {let data = new AudioData(invalid_init)},
      'AudioData requires numberOfFrames > 0');

  invalid_init = {...audio_data_init};
  invalid_init.numberOfChannels = 0

  assert_throws_js(
      TypeError, () => {let data = new AudioData(invalid_init)},
      'AudioData requires numberOfChannels > 0');

}, 'Verify AudioData constructors');

test(t => {
  let data = createDefaultAudioData();

  let clone = data.clone();

  // Verify the parameters match.
  assert_equals(data.timestamp, clone.timestamp, 'timestamp');
  assert_equals(data.numberOfFrames, clone.numberOfFrames, 'frames');
  assert_equals(data.numberOfChannels, clone.numberOfChannels, 'channels');
  assert_equals(data.sampleRate, clone.sampleRate, 'sampleRate');
  assert_equals(data.format, clone.format, 'format');

  const data_copyDest = new Float32Array(defaultInit.frames);
  const clone_copyDest = new Float32Array(defaultInit.frames);

  // Verify the data matches.
  for (var channel = 0; channel < defaultInit.channels; channel++) {
    data.copyTo(data_copyDest, {planeIndex: channel});
    clone.copyTo(clone_copyDest, {planeIndex: channel});

    assert_array_equals(
        data_copyDest, clone_copyDest, 'Cloned data ch=' + channel);
  }

  // Verify closing the original data doesn't close the clone.
  data.close();
  assert_equals(data.numberOfFrames, 0, 'data.buffer (closed)');
  assert_not_equals(clone.numberOfFrames, 0, 'clone.buffer (not closed)');

  clone.close();
  assert_equals(clone.numberOfFrames, 0, 'clone.buffer (closed)');

  // Verify closing a closed AudioData does not throw.
  data.close();
}, 'Verify closing and cloning AudioData');

test(t => {
  let data = make_audio_data(
      -10, defaultInit.channels, defaultInit.sampleRate, defaultInit.frames);
  assert_equals(data.timestamp, -10, 'timestamp');
  data.close();
}, 'Test we can construct AudioData with a negative timestamp.');


// Each test vector represents two channels of data in the following arbitrary
// layout: <min, zero, max, min, max / 2, min / 2, zero, max, zero, zero>.
const testVectorFrames = 5;
const testVectorChannels = 2;
const testVectorInterleavedResult =
    [[-1.0, 1.0, 0.5, 0.0, 0.0], [0.0, -1.0, -0.5, 1.0, 0.0]];
const testVectorPlanarResult =
    [[-1.0, 0.0, 1.0, -1.0, 0.5], [-0.5, 0.0, 1.0, 0.0, 0.0]];

test(t => {
  const INT8_MIN = (-0x7f - 1);
  const INT8_MAX = 0x7f;
  const UINT8_MAX = 0xff;

  const testVectorUint8 = [
    0, -INT8_MIN, UINT8_MAX, 0, INT8_MAX / 2 + 128, INT8_MIN / 2 + 128,
    -INT8_MIN, UINT8_MAX, -INT8_MIN, -INT8_MIN
  ];

  let data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Uint8Array(testVectorUint8),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'u8'
  });

  const epsilon = 1.0 / (UINT8_MAX - 1);

  let dest = new Float32Array(data.numberOfFrames);
  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[0], epsilon, 'interleaved channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[1], epsilon, 'interleaved channel 1');

  data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Uint8Array(testVectorUint8),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'u8-planar'
  });

  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[0], epsilon, 'planar channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[1], epsilon, 'planar channel 1');
}, 'Test conversion of uint8 data to float32');

test(t => {
  const INT16_MIN = (-0x7fff - 1);
  const INT16_MAX = 0x7fff;
  const testVectorInt16 = [
    INT16_MIN, 0, INT16_MAX, INT16_MIN, INT16_MAX / 2, INT16_MIN / 2, 0,
    INT16_MAX, 0, 0
  ];

  let data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Int16Array(testVectorInt16),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 's16'
  });

  const epsilon = 1.0 / (INT16_MAX + 1);

  let dest = new Float32Array(data.numberOfFrames);
  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[0], epsilon, 'interleaved channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[1], epsilon, 'interleaved channel 1');

  data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Int16Array(testVectorInt16),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 's16-planar'
  });

  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[0], epsilon, 'planar channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[1], epsilon, 'planar channel 1');
}, 'Test conversion of int16 data to float32');

test(t => {
  const INT32_MIN = (-0x7fffffff - 1);
  const INT32_MAX = 0x7fffffff;
  const testVectorInt32 = [
    INT32_MIN, 0, INT32_MAX, INT32_MIN, INT32_MAX / 2, INT32_MIN / 2, 0,
    INT32_MAX, 0, 0
  ];

  let data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Int32Array(testVectorInt32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 's32'
  });

  const epsilon = 1.0 / INT32_MAX;

  let dest = new Float32Array(data.numberOfFrames);
  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[0], epsilon, 'interleaved channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[1], epsilon, 'interleaved channel 1');

  data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Int32Array(testVectorInt32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 's32-planar'
  });

  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[0], epsilon, 'planar channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[1], epsilon, 'planar channel 1');
}, 'Test conversion of int32 data to float32');

test(t => {
  const testVectorFloat32 =
      [-1.0, 0.0, 1.0, -1.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0];

  let data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Float32Array(testVectorFloat32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32'
  });

  const epsilon = 0;

  let dest = new Float32Array(data.numberOfFrames);
  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[0], epsilon, 'interleaved channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorInterleavedResult[1], epsilon, 'interleaved channel 1');

  data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Float32Array(testVectorFloat32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32-planar'
  });

  data.copyTo(dest, {planeIndex: 0, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[0], epsilon, 'planar channel 0');
  data.copyTo(dest, {planeIndex: 1, format: 'f32-planar'});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[1], epsilon, 'planar channel 1');
}, 'Test conversion of float32 data to float32');

test(t => {
  const testVectorFloat32 =
      [-1.0, 0.0, 1.0, -1.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0];

  let data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Float32Array(testVectorFloat32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32'
  });

  const epsilon = 0;

  // Call copyTo() without specifying a format, for interleaved data.
  let dest = new Float32Array(data.numberOfFrames * testVectorChannels);
  data.copyTo(dest, {planeIndex: 0});
  assert_array_approx_equals(
      dest, testVectorFloat32, epsilon, 'interleaved data');

  assert_throws_js(RangeError, () => {
    data.copyTo(dest, {planeIndex: 1});
  }, 'Interleaved AudioData cannot copy out planeIndex > 0');

  data = new AudioData({
    timestamp: defaultInit.timestamp,
    data: new Float32Array(testVectorFloat32),
    numberOfFrames: testVectorFrames,
    numberOfChannels: testVectorChannels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32-planar'
  });

  // Call copyTo() without specifying a format, for planar data.
  dest = new Float32Array(data.numberOfFrames);
  data.copyTo(dest, {planeIndex: 0});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[0], epsilon, 'planar channel 0');
  data.copyTo(dest, {planeIndex: 1});
  assert_array_approx_equals(
      dest, testVectorPlanarResult[1], epsilon, 'planar channel 1');
}, 'Test copying out planar and interleaved data');
