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