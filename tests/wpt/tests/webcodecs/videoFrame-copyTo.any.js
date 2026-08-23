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
  const frame = makeNV12_4x2();
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

promise_test(async t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 8,
    codedHeight: 16,
    visibleRect: {
      x: 2,
      y: 2,
      width: 4,
      height: 8,
    },
    colorSpace: {
      primaries: 'smpte170m',
      transfer: 'smpte170m',
      matrix: 'smpte170m',
      fullRange: false,
    }
  };

  // Define YUV values for BT.601 red.
  const redY = 76;
  const redU = 84;
  const redV = 255;

  const ySize = init.codedWidth * init.codedHeight;
  const uvSize = ySize / 4;
  let data = new Uint8Array(ySize + 2 * uvSize);
  fillYUV(data, init.codedWidth, init.codedHeight, init.visibleRect, redY, redU,
          redV);

  let frame = new VideoFrame(data, init);
  assert_equals(frame.codedWidth, init.visibleRect.width);
  assert_equals(frame.codedHeight, init.visibleRect.height);
  assert_equals(frame.visibleRect.x, 0);
  assert_equals(frame.visibleRect.y, 0);
  assert_equals(frame.visibleRect.width, init.visibleRect.width);
  assert_equals(frame.visibleRect.height, init.visibleRect.height);
  assert_equals(frame.displayWidth, init.visibleRect.width);
  assert_equals(frame.displayHeight, init.visibleRect.height);

  let options = {rect: frame.visibleRect};
  let copied_data = new Uint8Array(frame.allocationSize(options));
  await frame.copyTo(copied_data, options);

  // We could write a bunch of code to carefully slice out the visible region
  // of `data`, but it's simpler and works better with testharness.js operators
  // to just create a fresh packed version for direct comparison.
  let packed_data = new Uint8Array(frame.allocationSize(options));
  fillYUV(packed_data, frame.codedWidth, frame.codedHeight, frame.visibleRect,
          redY, redU, redV);

  assert_array_equals(copied_data, packed_data, `Copied frame data incorrect.`);
}, 'copyTo from byte data with non-default visibleRect');

promise_test(async t => {
  const i420_4x4 = new Uint8Array([
    1,  2,  3,  4,   // y y y y
    5,  6,  7,  8,   // y y y y
    9,  10, 11, 12,  // y y y y
    13, 14, 15, 16,  // y y y y
    17, 18,          // u u
    19, 20,          // u u
    21, 22,          // v v
    23, 24,          // v v
  ]);
  const frame = new VideoFrame(i420_4x4, {
    format: 'I420',
    timestamp: 0,
    codedWidth: 4,
    codedHeight: 4,
  });

  const options = {
    layout: [
      {offset: 15, stride: 5},
      {offset: 1, stride: 3},
      {offset: 8, stride: 3},
    ],
  };
  const PAD = 0xAA;  // Distinct padding value so we can check for it later
  const expectedData = new Uint8Array([
    PAD,                    // unused
    17,  18, PAD,           // u u
    19,  20, PAD,           // u u
    PAD,                    // unused
    21,  22, PAD,           // v v
    23,  24, PAD,           // v v
    PAD,                    // unused
    1,   2,  3,   4,  PAD,  // y y y y
    5,   6,  7,   8,  PAD,  // y y y y
    9,   10, 11,  12, PAD,  // y y y y
    13,  14, 15,  16, PAD,  // y y y y
  ]);
  const size = frame.allocationSize(options);
  assert_equals(size, expectedData.length, 'allocationSize()');
  const data = new Uint8Array(size);
  data.fill(PAD);  // Initialize array with PAD before copying into it
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
}, 'Test I420 copyTo does not modify destination stride/offset padding bytes.');

promise_test(async t => {
  const W = 4;
  const H = 4;
  const src = new OffscreenCanvas(W, H);
  const g = src.getContext('2d');
  g.fillStyle = '#00ff00';
  g.fillRect(0, 0, W, H);  // Fill 4x4 source image with all green

  const frame = new VideoFrame(src, {timestamp: 0});

  const offset = 5;
  const stride = 20;
  const options = {
    layout: [
      {offset, stride},
    ],
  };
  const PAD = 0xAA;  // Distinct padding value so we can check for it later

  const size = frame.allocationSize(options);
  assert_equals(size, offset + stride * H, 'allocationSize()');

  const expectedData = new Uint8Array(size);
  expectedData.fill(PAD);
  for (let y = 0; y < H; ++y) {
    for (let x = 0; x < W; ++x) {
      expectedData[offset + y * stride + x * 4 + 0] = 0;    // R
      expectedData[offset + y * stride + x * 4 + 1] = 255;  // G
      expectedData[offset + y * stride + x * 4 + 2] = 0;    // B
      expectedData[offset + y * stride + x * 4 + 3] = 255;  // A
    }
  }

  const data = new Uint8Array(size);
  data.fill(PAD);  // Initialize array with PAD before copying into it
  const layout = await frame.copyTo(data, options);
  assert_layout_equals(layout, options.layout);
  assert_buffer_equals(data, expectedData);
  frame.close();
}, 'Test texture-backed RGBA copyTo does not modify destination stride/offset padding bytes.');
