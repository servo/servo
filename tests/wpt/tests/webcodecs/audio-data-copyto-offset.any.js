// META: global=window,dedicatedworker

test(t => {
  // 2 channels, 10 frames, interleaved f32.
  // Layout: [L0, R0, L1, R1, L2, R2, L3, R3, L4, R4,
  //          L5, R5, L6, R6, L7, R7, L8, R8, L9, R9]
  const channels = 2;
  const frames = 10;
  const data = new Float32Array(channels * frames);
  for (let i = 0; i < frames; i++) {
    data[i * channels] = i * 0.1;           // left
    data[i * channels + 1] = -(i * 0.1);    // right
  }

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const frameOffset = 3;
  const frameCount = 5;
  const dest = new Float32Array(channels * frameCount);
  audioData.copyTo(dest, {
    planeIndex: 0,
    format: 'f32',
    frameOffset: frameOffset,
    frameCount: frameCount,
  });

  // Expected: frames 3 through 7, interleaved.
  for (let i = 0; i < frameCount; i++) {
    const srcFrame = frameOffset + i;
    const expectedL = srcFrame * 0.1;
    const expectedR = -(srcFrame * 0.1);

    assert_approx_equals(dest[i * channels], expectedL, 1e-6,
        `frame ${srcFrame} left channel`);
    assert_approx_equals(dest[i * channels + 1], expectedR, 1e-6,
        `frame ${srcFrame} right channel`);
  }
}, 'copyTo interleaved-to-interleaved with non-zero frameOffset');

test(t => {
  // Same test with s16 to ensure it's not float-specific.
  const channels = 2;
  const frames = 10;
  const data = new Int16Array(channels * frames);
  for (let i = 0; i < frames; i++) {
    data[i * channels] = i * 1000;
    data[i * channels + 1] = -(i * 1000);
  }

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 's16',
  });
  t.add_cleanup(() => audioData.close());

  const frameOffset = 4;
  const frameCount = 3;
  const dest = new Int16Array(channels * frameCount);
  audioData.copyTo(dest, {
    planeIndex: 0,
    format: 's16',
    frameOffset: frameOffset,
    frameCount: frameCount,
  });

  for (let i = 0; i < frameCount; i++) {
    const srcFrame = frameOffset + i;
    assert_equals(dest[i * channels], srcFrame * 1000,
        `frame ${srcFrame} left channel`);
    assert_equals(dest[i * channels + 1], -(srcFrame * 1000),
        `frame ${srcFrame} right channel`);
  }
}, 'copyTo interleaved-to-interleaved s16 with non-zero frameOffset');

test(t => {
  // 3 channels, interleaved, to cover more than the stereo case.
  const channels = 3;
  const frames = 8;
  const data = new Float32Array(channels * frames);
  for (let i = 0; i < frames; i++) {
    for (let ch = 0; ch < channels; ch++) {
      data[i * channels + ch] = i + ch * 0.01;
    }
  }

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 'f32',
  });
  t.add_cleanup(() => audioData.close());

  const frameOffset = 2;
  const frameCount = 4;
  const dest = new Float32Array(channels * frameCount);
  audioData.copyTo(dest, {
    planeIndex: 0,
    format: 'f32',
    frameOffset: frameOffset,
    frameCount: frameCount,
  });

  for (let i = 0; i < frameCount; i++) {
    const srcFrame = frameOffset + i;
    for (let ch = 0; ch < channels; ch++) {
      const expected = srcFrame + ch * 0.01;
      assert_approx_equals(
          dest[i * channels + ch], expected, 1e-6,
          `frame ${srcFrame} channel ${ch}`);
    }
  }
}, 'copyTo interleaved-to-interleaved f32 with 3 channels and frameOffset');

test(t => {
  // 2 channels, 10 frames, planar f32.
  // Layout: [L0, L1, ..., L9, R0, R1, ..., R9]
  const channels = 2;
  const frames = 10;
  const data = new Float32Array(channels * frames);
  for (let ch = 0; ch < channels; ch++) {
    for (let i = 0; i < frames; i++) {
      data[ch * frames + i] = (ch + 1) * (i + 1) * 0.1;
    }
  }

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const frameOffset = 3;
  const frameCount = 5;
  const dest = new Float32Array(channels * frameCount);
  audioData.copyTo(dest, {
    planeIndex: 0,
    format: 'f32',
    frameOffset: frameOffset,
    frameCount: frameCount,
  });

  // Expected: frames 3 through 7 from each channel, interleaved.
  for (let i = 0; i < frameCount; i++) {
    const srcFrame = frameOffset + i;
    for (let ch = 0; ch < channels; ch++) {
      const expected = (ch + 1) * (srcFrame + 1) * 0.1;
      assert_approx_equals(
          dest[i * channels + ch], expected, 1e-5,
          `frame ${srcFrame} channel ${ch}`);
    }
  }
}, 'copyTo planar-to-interleaved with non-zero frameOffset');

test(t => {
  // 3 channels, planar s16.
  const channels = 3;
  const frames = 8;
  const data = new Int16Array(channels * frames);
  for (let ch = 0; ch < channels; ch++) {
    for (let i = 0; i < frames; i++) {
      data[ch * frames + i] = (ch + 1) * 1000 + i;
    }
  }

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 's16-planar',
  });
  t.add_cleanup(() => audioData.close());

  const frameOffset = 2;
  const frameCount = 4;
  const dest = new Int16Array(channels * frameCount);
  audioData.copyTo(dest, {
    planeIndex: 0,
    format: 's16',
    frameOffset: frameOffset,
    frameCount: frameCount,
  });

  for (let i = 0; i < frameCount; i++) {
    const srcFrame = frameOffset + i;
    for (let ch = 0; ch < channels; ch++) {
      const expected = (ch + 1) * 1000 + srcFrame;
      assert_equals(
          dest[i * channels + ch], expected,
          `frame ${srcFrame} channel ${ch}`);
    }
  }
}, 'copyTo planar-to-interleaved s16 with 3 channels and frameOffset');

test(t => {
  // Verify frameOffset=0 still works correctly for planar-to-interleaved
  // (baseline sanity check).
  const channels = 2;
  const frames = 4;
  const data = new Float32Array([
    1.0, 2.0, 3.0, 4.0,   // channel 0
    5.0, 6.0, 7.0, 8.0,   // channel 1
  ]);

  const audioData = new AudioData({
    timestamp: 0,
    data: data,
    numberOfFrames: frames,
    numberOfChannels: channels,
    sampleRate: 44100,
    format: 'f32-planar',
  });
  t.add_cleanup(() => audioData.close());

  const dest = new Float32Array(channels * frames);
  audioData.copyTo(dest, {planeIndex: 0, format: 'f32'});

  // Expected interleaved: [1, 5, 2, 6, 3, 7, 4, 8]
  const expected = new Float32Array([1, 5, 2, 6, 3, 7, 4, 8]);
  assert_array_equals(dest, expected,
      'planar-to-interleaved with frameOffset=0');
}, 'copyTo planar-to-interleaved baseline with frameOffset=0');
