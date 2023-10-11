// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-utils.js

function makeRGBA_2x2() {
  const data = new Uint8Array([
      1,2,3,4,    5,6,7,8,
      9,10,11,12, 13,14,15,16,
  ]);
  const init = {
      format: 'RGBA',
      timestamp: 0,
      codedWidth: 2,
      codedHeight: 2,
  };
  return new VideoFrame(data, init);
}

const NV12_DATA = new Uint8Array([
      1, 2, 3, 4,   // y
      5, 6, 7, 8,
      9, 10, 11, 12 // uv
  ]);

function makeNV12_4x2() {
  const init = {
      format: 'NV12',
      timestamp: 0,
      codedWidth: 4,
      codedHeight: 2,
  };
  return new VideoFrame(NV12_DATA, init);
}

promise_test(async t => {
  const frame = makeI420_4x2();
  frame.close();

  assert_throws_dom('InvalidStateError', () => frame.allocationSize(), 'allocationSize()');

  let data = new Uint8Array(12);
  await promise_rejects_dom(t, 'InvalidStateError', frame.copyTo(data), 'copyTo()');
}, 'Test closed frame.');

promise_test(async t => {
  const destination = new ArrayBuffer(I420_DATA.length);
  await testI420_4x2_copyTo(destination);
}, 'Test copying I420 frame to a non-shared ArrayBuffer');

promise_test(async t => {
  const destination = new Uint8Array(I420_DATA.length);
  await testI420_4x2_copyTo(destination);
}, 'Test copying I420 frame to a non-shared ArrayBufferView');

promise_test(async t => {
  const frame = makeRGBA_2x2();
  const expectedLayout = [
      {offset: 0, stride: 8},
  ];
  const expectedData = new Uint8Array([
      1,2,3,4,    5,6,7,8,
      9,10,11,12, 13,14,15,16,
  ]);
  assert_equals(frame.allocationSize(), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(data, expectedData);
}, 'Test RGBA frame.');

promise_test(async t => {
  const frame = makeNV12_4x2();
  const expectedLayout = [
      {offset: 0, stride: 4},
      {offset: 8, stride: 4},
  ];
  const expectedData = new Uint8Array([
      1,2,3,4,
      5,6,7,8,
      9,10,11,12
  ]);
  assert_equals(frame.allocationSize(), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(data, expectedData);
}, 'Test NV12 frame.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const data = new Uint8Array(11);
  await promise_rejects_js(t, TypeError, frame.copyTo(data));
}, 'Test undersized buffer.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
    layout: [{offset: 0, stride: 4}],
  };
  assert_throws_js(TypeError, () => frame.allocationSize(options));
  const data = new Uint8Array(12);
  await promise_rejects_js(t, TypeError, frame.copyTo(data, options));
}, 'Test incorrect plane count.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      layout: [
          {offset: 4, stride: 4},
          {offset: 0, stride: 2},
          {offset: 2, stride: 2},
      ],
  };
  const expectedData = new Uint8Array([
      9, 10,       // u
      11, 12,      // v
      1, 2, 3, 4,  // y
      5, 6, 7, 8,
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
}, 'Test I420 stride and offset work.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      layout: [
          {offset: 9, stride: 5},
          {offset: 1, stride: 3},
          {offset: 5, stride: 3},
      ],
  };
  const expectedData = new Uint8Array([
      0,
      9, 10, 0,       // u
      0,
      11, 12, 0,      // v
      0,
      1, 2, 3, 4, 0,  // y
      5, 6, 7, 8, 0,
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
}, 'Test I420 stride and offset with padding.');

promise_test(async t => {
  const init = {
    format: 'I420A',
    timestamp: 0,
    codedWidth: 4,
    codedHeight: 2,
  };
  const buf = new Uint8Array([
    1, 2, 3, 4,     // y
    5, 6, 7, 8,
    9, 10,          // u
    11, 12,         // v
    13, 14, 15, 16, // a
    17, 18, 19, 20,
  ]);
  const frame = new VideoFrame(buf, init);
  const options = {
      layout: [
          {offset: 12, stride: 4},
          {offset: 8, stride: 2},
          {offset: 10, stride: 2},
          {offset: 0, stride: 4},
      ],
  };
  const expectedData = new Uint8Array([
      13, 14, 15, 16, // a
      17, 18, 19, 20,
      9, 10,          // u
      11, 12,         // v
      1, 2, 3, 4,     // y
      5, 6, 7, 8,
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
}, 'Test I420A stride and offset work.');

promise_test(async t => {
  const init = {
    format: 'NV12',
    timestamp: 0,
    codedWidth: 4,
    codedHeight: 2,
  };
  const buf = new Uint8Array([
    1, 2, 3, 4,   // y
    5, 6, 7, 8,
    9, 10, 11, 12 // uv
  ]);
  const frame = new VideoFrame(buf, init);
  const options = {
      layout: [
          {offset: 4, stride: 4},
          {offset: 0, stride: 4},
      ],
  };
  const expectedData = new Uint8Array([
      9, 10, 11, 12, // uv
      1, 2, 3, 4,    // y
      5, 6, 7, 8
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
}, 'Test NV12 stride and offset work.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      layout: [
          {offset: 0, stride: 1},
          {offset: 8, stride: 2},
          {offset: 10, stride: 2},
      ],
  };
  assert_throws_js(TypeError, () => frame.allocationSize(options));
  const data = new Uint8Array(12);
  await promise_rejects_js(t, TypeError, frame.copyTo(data, options));
}, 'Test invalid stride.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      layout: [
          {offset: 0, stride: 4},
          {offset: 8, stride: 2},
          {offset: 2 ** 32 - 2, stride: 2},
      ],
  };
  assert_throws_js(TypeError, () => frame.allocationSize(options));
  const data = new Uint8Array(12);
  await promise_rejects_js(t, TypeError, frame.copyTo(data, options));
}, 'Test address overflow.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      rect: frame.codedRect,
  };
  const expectedLayout = [
      {offset: 0, stride: 4},
      {offset: 8, stride: 2},
      {offset: 10, stride: 2},
  ];
  const expectedData = new Uint8Array([
      1, 2, 3, 4, 5, 6, 7, 8,  // y
      9, 10,                   // u
      11, 12                   // v
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(data, expectedData);
}, 'Test codedRect.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      rect: {x: 0, y: 0, width: 4, height: 0},
  };
  assert_throws_js(TypeError, () => frame.allocationSize(options));
  const data = new Uint8Array(12);
  await promise_rejects_js(t, TypeError, frame.copyTo(data, options));
}, 'Test empty rect.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      rect: {x: 2, y: 0, width: 2, height: 2},
  };
  const expectedLayout = [
      {offset: 0, stride: 2},
      {offset: 4, stride: 1},
      {offset: 5, stride: 1},
  ];
  const expectedData = new Uint8Array([
      3, 4,  // y
      7, 8,
      10,    // u
      12     // v
  ]);
  assert_equals(frame.allocationSize(options), expectedData.length, 'allocationSize()');
  const data = new Uint8Array(expectedData.length);
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(data, expectedData);
}, 'Test left crop.');

promise_test(async t => {
  const frame = makeI420_4x2();
  const options = {
      rect: {x: 0, y: 0, width: 4, height: 4},
  };
  assert_throws_js(TypeError, () => frame.allocationSize(options));
  const data = new Uint8Array(12);
  await promise_rejects_js(t, TypeError, frame.copyTo(data, options));
}, 'Test invalid rect.');
