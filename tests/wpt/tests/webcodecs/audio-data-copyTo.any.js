// META: global=window,dedicatedworker

test(() => {
  for (const [formatFrom, formatTo] of [
    ['f32', 'f32'],
    ['f32', 'f32-planar'],
    ['f32-planar', 'f32-planar'],
    ['f32-planar', 'f32'],
  ]) {
    const data = new AudioData({
      format: formatFrom,
      sampleRate: 48000,
      numberOfChannels: 1,
      numberOfFrames: 5,
      data: new Float32Array([1, 2, 3, 4, 5]),
      timestamp: 0,
    });
    const output = new Float32Array(5);
    data.copyTo(output.subarray(1, 4), {
      planeIndex: 0,
      frameOffset: 1,
      frameCount: 3,
      format: formatTo,
    });
    data.close();
    assert_array_equals(output, [0, 2, 3, 4, 0], `only 3 middle elements are copied in ${formatFrom}->${formatTo} copy`);
  }
}, 'Test that AudioData.copyTo copies frameCount amount of mono frames from frameOffset');

test(() => {
  const data = new AudioData({
    format: 'f32-planar',
    sampleRate: 48000,
    numberOfChannels: 2,
    numberOfFrames: 5,
    data: new Float32Array([
      1, 2, 3, 4, 5, // left channel
      6, 7, 8, 9, 10, // right channel
    ]),
    timestamp: 0,
  });
  const output = new Float32Array(10);
  data.copyTo(output.subarray(1, 4), {
    planeIndex: 0,
    frameOffset: 1,
    frameCount: 3,
  });
  data.copyTo(output.subarray(data.numberOfFrames + 1, data.numberOfFrames + 4), {
    planeIndex: 1,
    frameOffset: 1,
    frameCount: 3,
  });
  data.close();
  assert_array_equals(output, [
    0, 2, 3, 4, 0, // left channel
    0, 7, 8, 9, 0, // right channel
  ], 'only 3 middle elements are copied in both channels');
}, 'Test that AudioData.copyTo copies frameCount amount of stereo frames from frameOffset');

test(() => {
  const data = new AudioData({
    format: 'f32',
    sampleRate: 48000,
    numberOfChannels: 2,
    numberOfFrames: 5,
    data: new Float32Array([
      1, 6,
      2, 7,
      3, 8,
      4, 9,
      5, 10,
    ]),
    timestamp: 0,
  });
  const output = new Float32Array(data.numberOfFrames * data.numberOfChannels);
  data.copyTo(output.subarray(2, 8), {
    planeIndex: 0,
    frameOffset: 1,
    frameCount: 3,
    format: 'f32',
  });
  data.close();
  assert_array_equals(output, [
    0, 0,
    2, 7,
    3, 8,
    4, 9,
    0, 0,
  ], 'only 3 middle elements are copied');
}, 'Test that AudioData.copyTo copies frameCount amount of interleaved stereo frames from frameOffset');

// Per the "Compute Copy Element Count" algorithm step 7
// (https://www.w3.org/TR/webcodecs/#compute-copy-element-count):
// "If options.frameOffset is greater than or equal to frameCount, throw a RangeError."
test(() => {
  const data = new AudioData({
    format: 'f32',
    sampleRate: 48000,
    numberOfChannels: 1,
    numberOfFrames: 5,
    data: new Float32Array([1, 2, 3, 4, 5]),
    timestamp: 0,
  });
  const output = new Float32Array(5);
  assert_throws_js(RangeError, () => {
    data.copyTo(output, { planeIndex: 0, frameOffset: 5 });
  }, 'frameOffset == numberOfFrames must throw RangeError');
  assert_throws_js(RangeError, () => {
    data.copyTo(output, { planeIndex: 0, frameOffset: 6 });
  }, 'frameOffset > numberOfFrames must throw RangeError');
  data.close();
}, 'AudioData.copyTo throws RangeError when frameOffset >= frameCount');
