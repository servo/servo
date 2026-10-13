// META: global=window,dedicatedworker

test(t => {
  // Interleaved to interleaved with oversized buffer.
  const frames = 4;
  const channels = 2;
  const source = new Float32Array(24);
  // Interleaved: [L0, R0, L1, R1, L2, R2, L3, R3]
  source[0] = 0.1; source[1] = 0.5;
  source[2] = 0.2; source[3] = 0.6;
  source[4] = 0.3; source[5] = 0.7;
  source[6] = 0.4; source[7] = 0.8;

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const dest = new Float32Array(frames * channels);
  audioData.copyTo(dest, {planeIndex: 0, format: 'f32'});

  const expected = new Float32Array([0.1, 0.5, 0.2, 0.6, 0.3, 0.7, 0.4, 0.8]);
  for (let i = 0; i < expected.length; i++) {
    assert_approx_equals(dest[i], expected[i], 0.00001,
        `sample ${i}`);
  }
}, 'copyTo interleaved-to-interleaved with oversized buffer');

test(t => {
  // Interleaved to interleaved with oversized buffer and frameOffset.
  const frames = 6;
  const channels = 2;
  const source = new Float32Array(frames * channels * 3);
  for (let i = 0; i < frames; i++) {
    source[i * channels] = i + 1;
    source[i * channels + 1] = i + 11;
  }

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const copyFrames = 3;
  const dest = new Float32Array(copyFrames * channels);
  audioData.copyTo(dest, {planeIndex: 0, format: 'f32',
                          frameOffset: 2, frameCount: copyFrames});

  const expected = new Float32Array([3, 13, 4, 14, 5, 15]);
  for (let i = 0; i < expected.length; i++) {
    assert_approx_equals(dest[i], expected[i], 0.00001,
        `sample ${i}`);
  }
}, 'copyTo interleaved-to-interleaved with oversized buffer and frameOffset');

test(t => {
  // Interleaved to planar with oversized buffer.
  const frames = 4;
  const channels = 2;
  const source = new Float32Array(24);
  for (let i = 0; i < frames; i++) {
    source[i * channels] = i + 1;
    source[i * channels + 1] = i + 11;
  }

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const dest0 = new Float32Array(frames);
  audioData.copyTo(dest0, {planeIndex: 0, format: 'f32-planar'});
  const expected0 = new Float32Array([1, 2, 3, 4]);
  for (let i = 0; i < expected0.length; i++) {
    assert_approx_equals(dest0[i], expected0[i], 0.00001, `ch0 frame ${i}`);
  }

  const dest1 = new Float32Array(frames);
  audioData.copyTo(dest1, {planeIndex: 1, format: 'f32-planar'});
  const expected1 = new Float32Array([11, 12, 13, 14]);
  for (let i = 0; i < expected1.length; i++) {
    assert_approx_equals(dest1[i], expected1[i], 0.00001, `ch1 frame ${i}`);
  }
}, 'copyTo interleaved-to-planar with oversized buffer');

test(t => {
  // Interleaved to planar with oversized buffer and frameOffset.
  const frames = 6;
  const channels = 2;
  const source = new Float32Array(frames * channels * 3);
  for (let i = 0; i < frames; i++) {
    source[i * channels] = i + 1;
    source[i * channels + 1] = i + 11;
  }

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const copyFrames = 3;
  const dest0 = new Float32Array(copyFrames);
  audioData.copyTo(dest0, {planeIndex: 0, format: 'f32-planar',
                           frameOffset: 2, frameCount: copyFrames});
  const expected0 = new Float32Array([3, 4, 5]);
  for (let i = 0; i < expected0.length; i++) {
    assert_approx_equals(dest0[i], expected0[i], 0.00001, `ch0 frame ${i}`);
  }

  const dest1 = new Float32Array(copyFrames);
  audioData.copyTo(dest1, {planeIndex: 1, format: 'f32-planar',
                           frameOffset: 2, frameCount: copyFrames});
  const expected1 = new Float32Array([13, 14, 15]);
  for (let i = 0; i < expected1.length; i++) {
    assert_approx_equals(dest1[i], expected1[i], 0.00001, `ch1 frame ${i}`);
  }
}, 'copyTo interleaved-to-planar with oversized buffer and frameOffset');

test(t => {
  // Planar to interleaved with oversized buffer.
  const frames = 4;
  const channels = 2;
  const source = new Float32Array(Math.ceil(frames * channels * 2.5));
  // ch0 at indices 0..3, ch1 at indices 4..7
  source[0] = 0.1; source[1] = 0.2; source[2] = 0.3; source[3] = 0.4;
  source[4] = 0.5; source[5] = 0.6; source[6] = 0.7; source[7] = 0.8;

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const dest = new Float32Array(frames * channels);
  audioData.copyTo(dest, {planeIndex: 0, format: 'f32'});

  const expected = new Float32Array([0.1, 0.5, 0.2, 0.6, 0.3, 0.7, 0.4, 0.8]);
  for (let i = 0; i < expected.length; i++) {
    assert_approx_equals(dest[i], expected[i], 0.00001, `sample ${i}`);
  }
}, 'copyTo planar-to-interleaved with oversized buffer');

test(t => {
  // Planar to interleaved with oversized buffer and frameOffset.
  const frames = 6;
  const channels = 2;
  const source = new Float32Array(frames * channels * 3);
  for (let i = 0; i < frames; i++) source[i] = i + 1;
  for (let i = 0; i < frames; i++) source[frames + i] = i + 11;

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const copyFrames = 4;
  const dest = new Float32Array(copyFrames * channels);
  audioData.copyTo(dest, {planeIndex: 0, format: 'f32',
                          frameOffset: 2, frameCount: copyFrames});

  const expected = new Float32Array([3, 13, 4, 14, 5, 15, 6, 16]);
  for (let i = 0; i < expected.length; i++) {
    assert_approx_equals(dest[i], expected[i], 0.00001, `sample ${i}`);
  }
}, 'copyTo planar-to-interleaved with oversized buffer and frameOffset');

test(t => {
  // Planar to planar with oversized buffer.
  const frames = 4;
  const channels = 2;
  const source = new Float32Array(frames * channels * 3);
  for (let i = 0; i < frames; i++) source[i] = i + 1;
  for (let i = 0; i < frames; i++) source[frames + i] = i + 11;

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const dest0 = new Float32Array(frames);
  audioData.copyTo(dest0, {planeIndex: 0, format: 'f32-planar'});
  const expected0 = new Float32Array([1, 2, 3, 4]);
  for (let i = 0; i < expected0.length; i++) {
    assert_approx_equals(dest0[i], expected0[i], 0.00001, `ch0 frame ${i}`);
  }

  const dest1 = new Float32Array(frames);
  audioData.copyTo(dest1, {planeIndex: 1, format: 'f32-planar'});
  const expected1 = new Float32Array([11, 12, 13, 14]);
  for (let i = 0; i < expected1.length; i++) {
    assert_approx_equals(dest1[i], expected1[i], 0.00001, `ch1 frame ${i}`);
  }
}, 'copyTo planar-to-planar with oversized buffer');

test(t => {
  // Planar to planar with oversized buffer and frameOffset.
  const frames = 6;
  const channels = 2;
  const source = new Float32Array(frames * channels * 3);
  for (let i = 0; i < frames; i++) source[i] = i + 1;
  for (let i = 0; i < frames; i++) source[frames + i] = i + 11;

  const audioData = new AudioData({
    data: source.buffer,
    numberOfFrames: frames,
    numberOfChannels: channels,
    timestamp: 0,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const copyFrames = 3;
  const dest0 = new Float32Array(copyFrames);
  audioData.copyTo(dest0, {planeIndex: 0, format: 'f32-planar',
                           frameOffset: 2, frameCount: copyFrames});
  const expected0 = new Float32Array([3, 4, 5]);
  for (let i = 0; i < expected0.length; i++) {
    assert_approx_equals(dest0[i], expected0[i], 0.00001, `ch0 frame ${i}`);
  }

  const dest1 = new Float32Array(copyFrames);
  audioData.copyTo(dest1, {planeIndex: 1, format: 'f32-planar',
                           frameOffset: 2, frameCount: copyFrames});
  const expected1 = new Float32Array([13, 14, 15]);
  for (let i = 0; i < expected1.length; i++) {
    assert_approx_equals(dest1[i], expected1[i], 0.00001, `ch1 frame ${i}`);
  }
}, 'copyTo planar-to-planar with oversized buffer and frameOffset');
