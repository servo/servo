const I420_DATA = new Uint8Array([
      1, 2, 3, 4,  // y
      5, 6, 7, 8,
      9, 10,       // u
      11, 12,      // v
  ]);

function makeI420_4x2() {
  const init = {
      format: 'I420',
      timestamp: 0,
      codedWidth: 4,
      codedHeight: 2,
  };
  return new VideoFrame(I420_DATA, init);
}

function testBufferConstructedI420Frame(bufferType) {
  let fmt = 'I420';
  let vfInit = {format: fmt, timestamp: 1234, codedWidth: 4, codedHeight: 2};

  let buffer;
  if (bufferType == 'SharedArrayBuffer' ||
      bufferType == 'Uint8Array(SharedArrayBuffer)') {
    buffer = new SharedArrayBuffer(I420_DATA.length);
  } else {
    assert_true(bufferType == 'ArrayBuffer' ||
                bufferType == 'Uint8Array(ArrayBuffer)');
    buffer = new ArrayBuffer(I420_DATA.length);
  }
  let bufferView = new Uint8Array(buffer);
  bufferView.set(I420_DATA);
  let data = bufferType.startsWith('Uint8Array') ? bufferView : buffer;

  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'bt709', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'bt709', 'color matrix');
  assert_false(frame.colorSpace.fullRange, 'color range');
  frame.close();

  let y = {offset: 0, stride: 4};
  let u = {offset: 8, stride: 2};
  let v = {offset: 10, stride: 2};

  assert_throws_js(TypeError, () => {
    let y = {offset: 0, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, u, v]});
  }, 'y stride too small');
  assert_throws_js(TypeError, () => {
    let u = {offset: 8, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, u, v]});
  }, 'u stride too small');
  assert_throws_js(TypeError, () => {
    let v = {offset: 10, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, u, v]});
  }, 'v stride too small');
  assert_throws_js(TypeError, () => {
    let frame = new VideoFrame(data.slice(0, 8), vfInit);
  }, 'data too small');
}

function assert_buffer_equals(actual, expected) {
  assert_true(expected instanceof Uint8Array, 'actual instanceof Uint8Array');

  if (actual instanceof ArrayBuffer ||
      (typeof(SharedArrayBuffer) != 'undefined' &&
        actual instanceof SharedArrayBuffer)) {
    actual = new Uint8Array(actual);
  } else {
    assert_true(actual instanceof Uint8Array,
        'expected instanceof Uint8Array, ArrayBuffer, or SharedArrayBuffer');
  }

  assert_equals(actual.length, expected.length, 'buffer length');
  for (let i = 0; i < actual.length; i++) {
    if (actual[i] != expected[i]) {
      assert_equals(actual[i], expected[i], 'buffer contents at index ' + i);
    }
  }
}

function assert_layout_equals(actual, expected) {
  assert_equals(actual.length, expected.length, 'layout planes');
  for (let i = 0; i < actual.length; i++) {
    assert_object_equals(actual[i], expected[i], 'plane ' + i + ' layout');
  }
}

async function testI420_4x2_copyTo(destination) {
  const frame = makeI420_4x2();
  const expectedLayout = [
      {offset: 0, stride: 4},
      {offset: 8, stride: 2},
      {offset: 10, stride: 2},
  ];
  const expectedData = new Uint8Array([
      1, 2, 3, 4,  // y
      5, 6, 7, 8,
      9, 10,       // u
      11, 12       // v
  ]);

  assert_equals(frame.allocationSize(), expectedData.length, 'allocationSize()');
  const layout = await frame.copyTo(destination);
  assert_layout_equals(layout, expectedLayout);
  assert_buffer_equals(destination, expectedData);
}

function verifyTimestampRequiredToConstructFrame(imageSource) {
  assert_throws_js(
      TypeError,
      () => new VideoFrame(imageSource),
      'timestamp required to construct VideoFrame from this source');
  let validFrame = new VideoFrame(imageSource, {timestamp: 0});
  validFrame.close();
}
