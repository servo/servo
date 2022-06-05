// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js
// META: script=/webcodecs/videoFrame-utils.js

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 10});

  assert_equals(frame.timestamp, 10, 'timestamp');
  assert_equals(frame.duration, null, 'duration');
  assert_equals(frame.visibleRect.width, 32, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 16, 'visibleRect.height');
  assert_equals(frame.displayWidth, 32, 'displayWidth');
  assert_equals(frame.displayHeight, 16, 'displayHeight');

  frame.close();
}, 'Test we can construct a VideoFrame.');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 10});
  frame.close();

  assert_equals(frame.format, null, 'format')
  assert_equals(frame.timestamp, null, 'timestamp');
  assert_equals(frame.duration, null, 'duration');
  assert_equals(frame.codedWidth, 0, 'codedWidth');
  assert_equals(frame.codedHeight, 0, 'codedHeight');
  assert_equals(frame.visibleRect, null, 'visibleRect');
  assert_equals(frame.displayWidth, 0, 'displayWidth');
  assert_equals(frame.displayHeight, 0, 'displayHeight');
  assert_equals(frame.colorSpace.primaries, null, 'colorSpace.primaries');
  assert_equals(frame.colorSpace.transfer, null, 'colorSpace.transfer');
  assert_equals(frame.colorSpace.matrix, null, 'colorSpace.matrix');
  assert_equals(frame.colorSpace.fullRange, null, 'colorSpace.fullRange');

  assert_throws_dom('InvalidStateError', () => frame.clone());
}, 'Test closed VideoFrame.');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: -10});
  assert_equals(frame.timestamp, -10, 'timestamp');
  frame.close();
}, 'Test we can construct a VideoFrame with a negative timestamp.');

promise_test(async t => {
  verifyTimestampRequiredToConstructFrame(makeImageBitmap(1, 1));
}, 'Test that timestamp is required when constructing VideoFrame from ImageBitmap');

promise_test(async t => {
  verifyTimestampRequiredToConstructFrame(makeOffscreenCanvas(16, 16));
}, 'Test that timestamp is required when constructing VideoFrame from OffscreenCanvas');

promise_test(async t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let i420Frame = new VideoFrame(data, init);
  let validFrame = new VideoFrame(i420Frame);
  validFrame.close();
}, 'Test that timestamp is NOT required when constructing VideoFrame from another VideoFrame');

test(t => {
  let image = makeImageBitmap(1, 1);
  let frame = new VideoFrame(image, {timestamp: 10});

  assert_equals(frame.visibleRect.width, 1, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 1, 'visibleRect.height');
  assert_equals(frame.displayWidth, 1, 'displayWidth');
  assert_equals(frame.displayHeight, 1, 'displayHeight');

  frame.close();
}, 'Test we can construct an odd-sized VideoFrame.');

test(t => {
  // Test only valid for Window contexts.
  if (!('document' in self))
    return;

  let video = document.createElement('video');

  assert_throws_dom('InvalidStateError', () => {
    let frame = new VideoFrame(video, {timestamp: 10});
  })
}, 'Test constructing w/ unusable image argument throws: HAVE_NOTHING <video>.');

test(t => {
  let canvas = new OffscreenCanvas(0, 0);

  assert_throws_dom('InvalidStateError', () => {
    let frame = new VideoFrame(canvas, {timestamp: 10});
  })
}, 'Test constructing w/ unusable image argument throws: emtpy Canvas.');

test(t => {
  let image = makeImageBitmap(32, 16);
  image.close();

  assert_throws_dom('InvalidStateError', () => {
    let frame = new VideoFrame(image, {timestamp: 10});
  })
}, 'Test constructing w/ unusable image argument throws: closed ImageBitmap.');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 10});
  frame.close();

  assert_throws_dom('InvalidStateError', () => {
    let newFrame = new VideoFrame(frame);
  })
}, 'Test constructing w/ unusable image argument throws: closed VideoFrame.');

test(t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let i420Frame = new VideoFrame(data, init);
  let image = makeImageBitmap(32, 16);


  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image,
          {timestamp: 10, visibleRect: {x: -1, y: 0, width: 10, height: 10}}),
      'negative visibleRect x');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image,
          {timestamp: 10, visibleRect: {x: 0, y: 0, width: -10, height: 10}}),
      'negative visibleRect width');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image,
          {timestamp: 10, visibleRect: {x: 0, y: 0, width: 10, height: 0}}),
      'zero visibleRect height');

  assert_throws_js(
      TypeError, () => new VideoFrame(image, {
                   timestamp: 10,
                   visibleRect: {x: 0, y: Infinity, width: 10, height: 10}
                 }),
      'non finite visibleRect y');

  assert_throws_js(
      TypeError, () => new VideoFrame(image, {
                   timestamp: 10,
                   visibleRect: {x: 0, y: 0, width: 10, height: Infinity}
                 }),
      'non finite visibleRect height');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image,
          {timestamp: 10, visibleRect: {x: 0, y: 0, width: 33, height: 17}}),
      'visibleRect area exceeds coded size');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image,
          {timestamp: 10, visibleRect: {x: 2, y: 2, width: 32, height: 16}}),
      'visibleRect outside coded size');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(image, {timestamp: 10, displayHeight: 10}),
      'displayHeight provided without displayWidth');

  assert_throws_js(
      TypeError, () => new VideoFrame(image, {timestamp: 10, displayWidth: 10}),
      'displayWidth provided without displayHeight');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image, {timestamp: 10, displayWidth: 0, displayHeight: 10}),
      'displayWidth is zero');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          image, {timestamp: 10, displayWidth: 10, displayHeight: 0}),
      'displayHeight is zero');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          i420Frame, {visibleRect: {x: 1, y: 0, width: 2, height: 2}}),
      'visibleRect x is not sample aligned');

  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          i420Frame, {visibleRect: {x: 0, y: 1, width: 2, height: 2}}),
      'visibleRect y is not sample aligned');

}, 'Test invalid CanvasImageSource constructed VideoFrames');

test(t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let origFrame = new VideoFrame(data, init);

  let cropLeftHalf = new VideoFrame(
      origFrame, {visibleRect: {x: 0, y: 0, width: 2, height: 2}});
  assert_equals(cropLeftHalf.codedWidth, origFrame.codedWidth);
  assert_equals(cropLeftHalf.codedHeight, origFrame.codedHeight);
  assert_equals(cropLeftHalf.visibleRect.x, 0);
  assert_equals(cropLeftHalf.visibleRect.y, 0);
  assert_equals(cropLeftHalf.visibleRect.width, 2);
  assert_equals(cropLeftHalf.visibleRect.height, 2);
  assert_equals(cropLeftHalf.displayWidth, 2);
  assert_equals(cropLeftHalf.displayHeight, 2);
}, 'Test visibleRect metadata override where source display size = visible size');

test(t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2,
    displayWidth: 8,
    displayHeight: 2
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let anamorphicFrame = new VideoFrame(data, init);

  let cropRightFrame = new VideoFrame(
      anamorphicFrame, {visibleRect: {x: 2, y: 0, width: 2, height: 2}});
  assert_equals(cropRightFrame.codedWidth, anamorphicFrame.codedWidth);
  assert_equals(cropRightFrame.codedHeight, anamorphicFrame.codedHeight);
  assert_equals(cropRightFrame.visibleRect.x, 2);
  assert_equals(cropRightFrame.visibleRect.y, 0);
  assert_equals(cropRightFrame.visibleRect.width, 2);
  assert_equals(cropRightFrame.visibleRect.height, 2);
  assert_equals(cropRightFrame.displayWidth, 4, 'cropRightFrame.displayWidth');
  assert_equals(cropRightFrame.displayHeight, 2, 'cropRightFrame.displayHeight');
}, 'Test visibleRect metadata override where source display width = 2 * visible width (anamorphic)');

test(t => {
  let init = {
    format: 'I420',
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2,
    displayWidth: 8,
    displayHeight: 4
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let scaledFrame = new VideoFrame(data, init);

  let cropRightFrame = new VideoFrame(
      scaledFrame, {visibleRect: {x: 2, y: 0, width: 2, height: 2}});
  assert_equals(cropRightFrame.codedWidth, scaledFrame.codedWidth);
  assert_equals(cropRightFrame.codedHeight, scaledFrame.codedHeight);
  assert_equals(cropRightFrame.visibleRect.x, 2);
  assert_equals(cropRightFrame.visibleRect.y, 0);
  assert_equals(cropRightFrame.visibleRect.width, 2);
  assert_equals(cropRightFrame.visibleRect.height, 2);
  assert_equals(cropRightFrame.displayWidth, 4, 'cropRightFrame.displayWidth');
  assert_equals(cropRightFrame.displayHeight, 4, 'cropRightFrame.displayHeight');
}, 'Test visibleRect metadata override where source display size = 2 * visible size for both width and height');

test(t => {
  let image = makeImageBitmap(32, 16);

  let scaledFrame = new VideoFrame(image, {
    visibleRect: {x: 0, y: 0, width: 2, height: 2},
    displayWidth: 10,
    displayHeight: 20,
    timestamp: 0
  });
  assert_equals(scaledFrame.codedWidth, 32);
  assert_equals(scaledFrame.codedHeight, 16);
  assert_equals(scaledFrame.visibleRect.x, 0);
  assert_equals(scaledFrame.visibleRect.y, 0);
  assert_equals(scaledFrame.visibleRect.width, 2);
  assert_equals(scaledFrame.visibleRect.height, 2);
  assert_equals(scaledFrame.displayWidth, 10, 'scaledFrame.displayWidth');
  assert_equals(scaledFrame.displayHeight, 20, 'scaledFrame.displayHeight');
}, 'Test visibleRect + display size metadata override');

test(t => {
  let image = makeImageBitmap(32, 16);

  let scaledFrame = new VideoFrame(image,
    {
      displayWidth: 10, displayHeight: 20,
      timestamp: 0
    });
  assert_equals(scaledFrame.codedWidth, 32);
  assert_equals(scaledFrame.codedHeight, 16);
  assert_equals(scaledFrame.visibleRect.x, 0);
  assert_equals(scaledFrame.visibleRect.y, 0);
  assert_equals(scaledFrame.visibleRect.width, 32);
  assert_equals(scaledFrame.visibleRect.height, 16);
  assert_equals(scaledFrame.displayWidth, 10, 'scaledFrame.displayWidth');
  assert_equals(scaledFrame.displayHeight, 20, 'scaledFrame.displayHeight');
}, 'Test display size metadata override');

test(t => {
  assert_throws_js(
      TypeError,
      () => new VideoFrame(
          new Uint8Array(1),
          {format: 'ABCD', timestamp: 1234, codedWidth: 4, codedHeight: 2}),
      'invalid pixel format');

  assert_throws_js(
      TypeError,
      () =>
          new VideoFrame(new Uint32Array(1), {format: 'RGBA', timestamp: 1234}),
      'missing coded size');

  function constructFrame(init) {
    let data = new Uint8Array([
      1, 2, 3, 4, 5, 6, 7, 8,  // y
      1, 2,                    // u
      1, 2,                    // v
    ]);
    return new VideoFrame(data, {...init, format: 'I420'});
  }

  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: Math.pow(2, 32) - 1,
                   codedHeight: Math.pow(2, 32) - 1,
                 }),
      'invalid coded size');
  assert_throws_js(
      TypeError,
      () => constructFrame({timestamp: 1234, codedWidth: 4, codedHeight: 0}),
      'invalid coded height');
  assert_throws_js(
      TypeError,
      () => constructFrame({timestamp: 1234, codedWidth: 4, codedHeight: 1}),
      'odd coded height');
  assert_throws_js(
      TypeError,
      () => constructFrame({timestamp: 1234, codedWidth: 0, codedHeight: 4}),
      'invalid coded width');
  assert_throws_js(
      TypeError,
      () => constructFrame({timestamp: 1234, codedWidth: 3, codedHeight: 2}),
      'odd coded width');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   visibleRect: {x: 100, y: 100, width: 1, height: 1}
                 }),
      'invalid visible left/right');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   visibleRect: {x: 0, y: 0, width: 0, height: 2}
                 }),
      'invalid visible width');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   visibleRect: {x: 0, y: 0, width: 2, height: 0}
                 }),
      'invalid visible height');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   visibleRect: {x: 0, y: 0, width: -100, height: -100}
                 }),
      'invalid negative visible size');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   displayWidth: Math.pow(2, 32),
                 }),
      'invalid display width');
  assert_throws_js(
      TypeError, () => constructFrame({
                   timestamp: 1234,
                   codedWidth: 4,
                   codedHeight: 2,
                   displayWidth: Math.pow(2, 32) - 1,
                   displayHeight: Math.pow(2, 32)
                 }),
      'invalid display height');
}, 'Test invalid buffer constructed VideoFrames');

test(t => {
  testBufferConstructedI420Frame('Uint8Array(ArrayBuffer)');
}, 'Test Uint8Array(ArrayBuffer) constructed I420 VideoFrame');

test(t => {
  testBufferConstructedI420Frame('ArrayBuffer');
}, 'Test ArrayBuffer constructed I420 VideoFrame');

test(t => {
  let fmt = 'I420';
  let vfInit = {
    format: fmt,
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2,
    colorSpace: {
      primaries: 'smpte170m',
      matrix: 'bt470bg',
    },
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.colorSpace.primaries, 'smpte170m', 'color primaries');
  assert_true(frame.colorSpace.transfer == null, 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'bt470bg', 'color matrix');
  assert_true(frame.colorSpace.fullRange == null, 'color range');
}, 'Test planar constructed I420 VideoFrame with colorSpace');

test(t => {
  let fmt = 'I420A';
  let vfInit = {format: fmt, timestamp: 1234, codedWidth: 4, codedHeight: 2};
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
    8, 7, 6, 5, 4, 3, 2, 1,  // a
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'bt709', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'bt709', 'color matrix');
  assert_false(frame.colorSpace.fullRange, 'color range');
  frame.close();

  // Most constraints are tested as part of I420 above.

  let y = {offset: 0, stride: 4};
  let u = {offset: 8, stride: 2};
  let v = {offset: 10, stride: 2};
  let a = {offset: 12, stride: 4};

  assert_throws_js(TypeError, () => {
    let a = {offset: 12, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, u, v, a]});
  }, 'a stride too small');
  assert_throws_js(TypeError, () => {
    let frame = new VideoFrame(data.slice(0, 12), vfInit);
  }, 'data too small');
}, 'Test buffer constructed I420+Alpha VideoFrame');

test(t => {
  let fmt = 'NV12';
  let vfInit = {format: fmt, timestamp: 1234, codedWidth: 4, codedHeight: 2};
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2, 3, 4,              // uv
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'bt709', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'bt709', 'color matrix');
  assert_false(frame.colorSpace.fullRange, 'color range');
  frame.close();

  let y = {offset: 0, stride: 4};
  let uv = {offset: 8, stride: 4};

  assert_throws_js(TypeError, () => {
    let y = {offset: 0, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, uv]});
  }, 'y stride too small');
  assert_throws_js(TypeError, () => {
    let uv = {offset: 8, stride: 1};
    let frame = new VideoFrame(data, {...vfInit, layout: [y, uv]});
  }, 'uv stride too small');
  assert_throws_js(TypeError, () => {
    let frame = new VideoFrame(data.slice(0, 8), vfInit);
  }, 'data too small');
}, 'Test buffer constructed NV12 VideoFrame');

test(t => {
  let vfInit = {timestamp: 1234, codedWidth: 4, codedHeight: 2};
  let data = new Uint8Array([
    1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
  ]);
  let frame = new VideoFrame(data, {...vfInit, format: 'RGBA'});
  assert_equals(frame.format, 'RGBA', 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'iec61966-2-1', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'rgb', 'color matrix');
  assert_true(frame.colorSpace.fullRange, 'color range');
  frame.close();

  frame = new VideoFrame(data, {...vfInit, format: 'RGBX'});
  assert_equals(frame.format, 'RGBX', 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'iec61966-2-1', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'rgb', 'color matrix');
  assert_true(frame.colorSpace.fullRange, 'color range');
  frame.close();

  frame = new VideoFrame(data, {...vfInit, format: 'BGRA'});
  assert_equals(frame.format, 'BGRA', 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'iec61966-2-1', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'rgb', 'color matrix');
  assert_true(frame.colorSpace.fullRange, 'color range');
  frame.close();

  frame = new VideoFrame(data, {...vfInit, format: 'BGRX'});
  assert_equals(frame.format, 'BGRX', 'plane format');
  assert_equals(frame.colorSpace.primaries, 'bt709', 'color primaries');
  assert_equals(frame.colorSpace.transfer, 'iec61966-2-1', 'color transfer');
  assert_equals(frame.colorSpace.matrix, 'rgb', 'color matrix');
  assert_true(frame.colorSpace.fullRange, 'color range');
  frame.close();
}, 'Test buffer constructed RGB VideoFrames');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 0});
  assert_true(!!frame);

  frame_copy = new VideoFrame(frame);
  assert_equals(frame.format, frame_copy.format);
  assert_equals(frame.timestamp, frame_copy.timestamp);
  assert_equals(frame.codedWidth, frame_copy.codedWidth);
  assert_equals(frame.codedHeight, frame_copy.codedHeight);
  assert_equals(frame.displayWidth, frame_copy.displayWidth);
  assert_equals(frame.displayHeight, frame_copy.displayHeight);
  assert_equals(frame.duration, frame_copy.duration);
  frame_copy.close();

  frame_copy = new VideoFrame(frame, {duration: 1234});
  assert_equals(frame.timestamp, frame_copy.timestamp);
  assert_equals(frame_copy.duration, 1234);
  frame_copy.close();

  frame_copy = new VideoFrame(frame, {timestamp: 1234, duration: 456});
  assert_equals(frame_copy.timestamp, 1234);
  assert_equals(frame_copy.duration, 456);
  frame_copy.close();

  frame.close();
}, 'Test VideoFrame constructed VideoFrame');

test(t => {
  let canvas = makeOffscreenCanvas(16, 16);
  let frame = new VideoFrame(canvas, {timestamp: 0});
  assert_equals(frame.displayWidth, 16);
  assert_equals(frame.displayHeight, 16);
  frame.close();
}, 'Test we can construct a VideoFrame from an offscreen canvas.');

test(t => {
  let fmt = 'I420';
  let vfInit = {
    format: fmt,
    timestamp: 1234,
    codedWidth: 4,
    codedHeight: 2,
    visibleRect: {x: 0, y: 0, width: 1, height: 1},
  };
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
    8, 7, 6, 5, 4, 3, 2, 1,  // a
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'format');
  assert_equals(frame.visibleRect.x, 0, 'visibleRect.x');
  assert_equals(frame.visibleRect.y, 0, 'visibleRect.y');
  assert_equals(frame.visibleRect.width, 1, 'visibleRect.width');
  assert_equals(frame.visibleRect.height, 1, 'visibleRect.height');
  frame.close();
}, 'Test I420 VideoFrame with odd visible size');

test(t => {
  let fmt = 'I420A';
  let vfInit = {format: fmt, timestamp: 1234, codedWidth: 4, codedHeight: 2};
  let data = new Uint8Array([
    1, 2, 3, 4, 5, 6, 7, 8,  // y
    1, 2,                    // u
    1, 2,                    // v
    8, 7, 6, 5, 4, 3, 2, 1,  // a
  ]);
  let frame = new VideoFrame(data, vfInit);
  assert_equals(frame.format, fmt, 'plane format');

  let alpha_frame_copy = new VideoFrame(frame, {alpha: 'keep'});
  assert_equals(alpha_frame_copy.format, 'I420A', 'plane format');

  let opaque_frame_copy = new VideoFrame(frame, {alpha: 'discard'});
  assert_equals(opaque_frame_copy.format, 'I420', 'plane format');

  frame.close();
  alpha_frame_copy.close();
  opaque_frame_copy.close();
}, 'Test I420A VideoFrame and alpha={keep,discard}');

test(t => {
  let vfInit = {timestamp: 1234, codedWidth: 4, codedHeight: 2};
  let data = new Uint8Array([
    1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
  ]);
  let frame = new VideoFrame(data, {...vfInit, format: 'RGBA'});
  assert_equals(frame.format, 'RGBA', 'plane format');

  let alpha_frame_copy = new VideoFrame(frame, {alpha: 'keep'});
  assert_equals(alpha_frame_copy.format, 'RGBA', 'plane format');

  let opaque_frame_copy = new VideoFrame(frame, {alpha: 'discard'});
  assert_equals(opaque_frame_copy.format, 'RGBX', 'plane format');

  alpha_frame_copy.close();
  opaque_frame_copy.close();
  frame.close();

  frame = new VideoFrame(data, {...vfInit, format: 'BGRA'});
  assert_equals(frame.format, 'BGRA', 'plane format');

  alpha_frame_copy = new VideoFrame(frame, {alpha: 'keep'});
  assert_equals(alpha_frame_copy.format, 'BGRA', 'plane format');

  opaque_frame_copy = new VideoFrame(frame, {alpha: 'discard'});
  assert_equals(opaque_frame_copy.format, 'BGRX', 'plane format');

  alpha_frame_copy.close();
  opaque_frame_copy.close();
  frame.close();
}, 'Test RGBA, BGRA VideoFrames with alpha={keep,discard}');

test(t => {
  let canvas = makeOffscreenCanvas(16, 16, {alpha: true});
  let frame = new VideoFrame(canvas, {timestamp: 0});
  assert_true(
      frame.format == 'RGBA' || frame.format == 'BGRA' ||
          frame.format == 'I420A',
      'plane format should have alpha: ' + frame.format);
  frame.close();

  frame = new VideoFrame(canvas, {alpha: 'discard', timestamp: 0});
  assert_true(
      frame.format == 'RGBX' || frame.format == 'BGRX' ||
          frame.format == 'I420',
      'plane format should not have alpha: ' + frame.format);
  frame.close();
}, 'Test a VideoFrame constructed from canvas can drop the alpha channel.');

