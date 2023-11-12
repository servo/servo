// META: global=window,dedicatedworker

promise_test(async t => {
  let fmt = 'RGBA';
  const rgb_plane = [
    0xBA, 0xDF, 0x00, 0xD0, 0xBA, 0xDF, 0x01, 0xD0, 0xBA, 0xDF, 0x02, 0xD0,
    0xBA, 0xDF, 0x03, 0xD0
  ];
  let data = new Uint8Array(rgb_plane);
  let unused_buffer = new ArrayBuffer(123);
  let init = {
    format: fmt,
    timestamp: 1234,
    codedWidth: 2,
    codedHeight: 2,
    visibleRect: {x: 0, y: 0, width: 2, height: 2},
    transfer: [data.buffer, unused_buffer]
  };
  assert_equals(data.length, 16, 'data.length');
  assert_equals(unused_buffer.byteLength, 123, 'unused_buffer.byteLength');

  let frame = new VideoFrame(data, init);
  assert_equals(frame.format, fmt, 'format');
  assert_equals(data.length, 0, 'data.length after detach');
  assert_equals(unused_buffer.byteLength, 0, 'unused_buffer after detach');

  const options = {
    rect: {x: 0, y: 0, width: init.codedWidth, height: init.codedHeight}
  };
  let size = frame.allocationSize(options);
  let output_data = new Uint8Array(size);
  let layout = await frame.copyTo(output_data, options);
  let expected_data = new Uint8Array(rgb_plane);
  assert_equals(expected_data.length, size, 'expected_data size');
  for (let i = 0; i < size; i++) {
    assert_equals(expected_data[i], output_data[i], `expected_data[${i}]`);
  }

  frame.close();
}, 'Test transfering ArrayBuffer to VideoFrame');


promise_test(async t => {
  const rgb_plane = [
    0xBA, 0xDF, 0x00, 0xD0, 0xBA, 0xDF, 0x01, 0xD0, 0xBA, 0xDF, 0x02, 0xD0,
    0xBA, 0xDF, 0x03, 0xD0
  ];
  let data = new Uint8Array(rgb_plane);
  let detached_buffer = new ArrayBuffer(123);

  // Detach `detached_buffer`
  structuredClone({x: detached_buffer}, {transfer: [detached_buffer]});

  let init = {
    format: 'RGBA',
    timestamp: 1234,
    codedWidth: 2,
    codedHeight: 2,
    visibleRect: {x: 0, y: 0, width: 2, height: 2},
    transfer: [data.buffer, detached_buffer]
  };

  try {
    new VideoFrame(data, init);
  } catch (error) {
    assert_equals(error.name, 'DataCloneError', 'error.name');
  }
  // `data.buffer` didn't get detached
  assert_equals(data.length, 16, 'data.length');
}, 'Test transfering detached buffer to VideoFrame');


promise_test(async t => {
  const rgb_plane = [
    0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE,
    0xEE, 0xEE, 0xEE, 0xEE
  ];
  const padding_size = 6;
  let arraybuffer = new ArrayBuffer(padding_size + 16 /* pixels */);
  let data = new Uint8Array(arraybuffer, padding_size);
  data.set(rgb_plane);

  let init = {
    format: 'RGBA',
    timestamp: 1234,
    codedWidth: 2,
    codedHeight: 2,
    visibleRect: {x: 0, y: 0, width: 2, height: 2},
    transfer: [arraybuffer]
  };

  let frame = new VideoFrame(data, init);
  assert_equals(data.length, 0, 'data.length after detach');
  assert_equals(arraybuffer.byteLength, 0, 'arraybuffer after detach');

  const options = {
    rect: {x: 0, y: 0, width: init.codedWidth, height: init.codedHeight}
  };
  let size = frame.allocationSize(options);
  let output_data = new Uint8Array(size);
  let layout = await frame.copyTo(output_data, options);
  for (let i = 0; i < size; i++) {
    assert_equals(output_data[i], 0xEE, `output_data[${i}]`);
  }
}, 'Test transfering view of an ArrayBuffer to VideoFrame');

promise_test(async t => {
  const rgb_plane = [
    0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE,
    0xEE, 0xEE, 0xEE, 0xEE
  ];
  const padding_size = 6;
  let arraybuffer = new ArrayBuffer(padding_size + 16 /* pixels */);
  let data = new Uint8Array(arraybuffer, padding_size);
  data.set(rgb_plane);

  let init = {
    format: 'RGBA',
    timestamp: 1234,
    codedWidth: 2,
    codedHeight: 2,
    visibleRect: {x: 0, y: 0, width: 2, height: 2},
    transfer: [arraybuffer, arraybuffer]
  };

  try {
    new VideoFrame(data, init);
  } catch (error) {
    assert_equals(error.name, 'DataCloneError', 'error.name');
  }
  // `data.buffer` didn't get detached
  assert_equals(data.length, 16, 'data.length');
}, 'Test transfering same array buffer twice');

promise_test(async t => {
  const bytes = [ 0xBA, 0xDF, 0x00, 0xD0, 0xBA, 0xDF, 0x01, 0xD0, 0xBA, 0xDF ];
  let data = new Uint8Array(bytes);
  let unused_buffer = new ArrayBuffer(123);
  let init = {
    type: 'key',
    timestamp: 0,
    data: data,
    transfer: [data.buffer, unused_buffer]
  };

  assert_equals(data.length, 10, 'data.length');
  assert_equals(unused_buffer.byteLength, 123, 'unused_buffer.byteLength');

  let chunk = new EncodedAudioChunk(init);
  assert_equals(data.length, 0, 'data.length after detach');
  assert_equals(unused_buffer.byteLength, 0, 'unused_buffer after detach');

  let output_data = new Uint8Array(chunk.byteLength);
  chunk.copyTo(output_data);
  let expected_data = new Uint8Array(bytes);
  assert_equals(expected_data.length, chunk.byteLength, 'expected_data size');
  for (let i = 0; i < chunk.byteLength; i++) {
    assert_equals(expected_data[i], output_data[i], `expected_data[${i}]`);
  }
}, 'Test transfering ArrayBuffer to EncodedAudioChunk');

promise_test(async t => {
  const bytes = [ 0xBA, 0xDF, 0x00, 0xD0, 0xBA, 0xDF, 0x01, 0xD0, 0xBA, 0xDF ];
  let data = new Uint8Array(bytes);
  let unused_buffer = new ArrayBuffer(123);
  let init = {
    type: 'key',
    timestamp: 0,
    data: data,
    transfer: [data.buffer, unused_buffer]
  };

  assert_equals(data.length, 10, 'data.length');
  assert_equals(unused_buffer.byteLength, 123, 'unused_buffer.byteLength');

  let chunk = new EncodedVideoChunk(init);
  assert_equals(data.length, 0, 'data.length after detach');
  assert_equals(unused_buffer.byteLength, 0, 'unused_buffer after detach');

  let output_data = new Uint8Array(chunk.byteLength);
  chunk.copyTo(output_data);
  let expected_data = new Uint8Array(bytes);
  assert_equals(expected_data.length, chunk.byteLength, 'expected_data size');
  for (let i = 0; i < chunk.byteLength; i++) {
    assert_equals(expected_data[i], output_data[i], `expected_data[${i}]`);
  }
}, 'Test transfering ArrayBuffer to EncodedVideoChunk');

promise_test(async t => {
  const bytes = [0xBA, 0xDF, 0x00, 0xD0, 0xBA, 0xDF, 0x01, 0xD0, 0xBA, 0xDF];
  let data = new Uint8Array(bytes);
  let unused_buffer = new ArrayBuffer(123);
  let init = {
    type: 'key',
    timestamp: 0,
    numberOfFrames: data.length,
    numberOfChannels: 1,
    sampleRate: 10000,
    format: 'u8',
    data: data,
    transfer: [data.buffer, unused_buffer]
  };

  assert_equals(data.length, 10, 'data.length');
  assert_equals(unused_buffer.byteLength, 123, 'unused_buffer.byteLength');

  let audio_data = new AudioData(init);
  assert_equals(data.length, 0, 'data.length after detach');
  assert_equals(unused_buffer.byteLength, 0, 'unused_buffer after detach');

  let readback_data = new Uint8Array(bytes.length);
  audio_data.copyTo(readback_data, {planeIndex: 0, format: 'u8'});
  let expected_data = new Uint8Array(bytes);
  for (let i = 0; i < expected_data.length; i++) {
    assert_equals(expected_data[i], readback_data[i], `expected_data[${i}]`);
  }
}, 'Test transfering ArrayBuffer to AudioData');

promise_test(async t => {
  let sample_rate = 48000;
  let total_duration_s = 1;
  let data_count = 10;
  let chunks = [];

  let encoder_init = {
    error: t.unreached_func('Encoder error'),
    output: (chunk, metadata) => {
      chunks.push(chunk);
    }
  };
  let encoder = new AudioEncoder(encoder_init);
  let config = {
    codec: 'opus',
    sampleRate: sample_rate,
    numberOfChannels: 2,
    bitrate: 256000,  // 256kbit
  };
  encoder.configure(config);

  let timestamp_us = 0;
  const data_duration_s = total_duration_s / data_count;
  const frames = data_duration_s * config.sampleRate;
  for (let i = 0; i < data_count; i++) {
    let buffer = new Float32Array(frames * config.numberOfChannels);
    let data = new AudioData({
      timestamp: timestamp_us,
      data: buffer,
      numberOfChannels: config.numberOfChannels,
      numberOfFrames: frames,
      sampleRate: config.sampleRate,
      format: 'f32-planar',
      transfer: [buffer.buffer]
    });
    timestamp_us += data_duration_s * 1_000_000;
    assert_equals(buffer.length, 0, 'buffer.length after detach');
    encoder.encode(data);
  }
  await encoder.flush();
  encoder.close();
  assert_greater_than(chunks.length, 0);
}, 'Encoding from AudioData with transferred buffer');


promise_test(async t => {
  let unused_buffer = new ArrayBuffer(123);
  let support = await ImageDecoder.isTypeSupported('image/png');
  assert_implements_optional(
      support, 'Optional codec image/png not supported.');
  let buffer = await fetch('four-colors.png').then(response => {
    return response.arrayBuffer();
  });

  let decoder = new ImageDecoder(
      {data: buffer, type: 'image/png', transfer: [buffer, unused_buffer]});
  assert_equals(buffer.byteLength, 0, 'buffer.byteLength after detach');
  assert_equals(unused_buffer.byteLength, 0, 'unused_buffer after detach');

  let result = await decoder.decode();
  assert_equals(result.image.displayWidth, 320);
  assert_equals(result.image.displayHeight, 240);
}, 'Test transfering ArrayBuffer to ImageDecoder.');
