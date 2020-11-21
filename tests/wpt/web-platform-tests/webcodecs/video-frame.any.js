// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 10});

  assert_equals(frame.timestamp, 10, "timestamp");
  assert_equals(frame.duration, null, "duration");
  assert_equals(frame.cropWidth, 32, "cropWidth");
  assert_equals(frame.cropHeight, 16, "cropHeight");
  assert_equals(frame.cropWidth, 32, "displayWidth");
  assert_equals(frame.cropHeight, 16, "displayHeight");

  frame.destroy();
}, 'Test we can construct a VideoFrame.');

test(t => {
  let image = makeImageBitmap(1, 1);
  let frame = new VideoFrame(image, {timestamp: 10});

  assert_equals(frame.cropWidth, 1, "cropWidth");
  assert_equals(frame.cropHeight, 1, "cropHeight");
  assert_equals(frame.cropWidth, 1, "displayWidth");
  assert_equals(frame.cropHeight, 1, "displayHeight");

  frame.destroy();
}, 'Test we can construct an odd-sized VideoFrame.');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 0});

  // TODO(sandersd): This would be more clear as RGBA, but conversion has
  // not be specified (or implemented) yet.
  if (frame.format !== "I420") {
    return;
  }
  assert_equals(frame.planes.length, 3, "number of planes");

  // Validate Y plane metadata.
  let yPlane = frame.planes[0];
  let yStride = yPlane.stride;
  let yRows = yPlane.rows;
  let yLength = yPlane.length;

  // Required minimums to contain the visible data.
  assert_greater_than_equal(yRows, 16, "Y plane rows");
  assert_greater_than_equal(yStride, 32, "Y plane stride");
  assert_greater_than_equal(yLength, 32 * 16, "Y plane length");

  // Not required by spec, but sets limit at 50% padding per dimension.
  assert_less_than_equal(yRows, 32, "Y plane rows");
  assert_less_than_equal(yStride, 64, "Y plane stride");
  assert_less_than_equal(yLength, 32 * 64, "Y plane length");

  // Validate Y plane data.
  let buffer = new ArrayBuffer(yLength);
  let view = new Uint8Array(buffer);
  frame.planes[0].readInto(view);

  // TODO(sandersd): This probably needs to be fuzzy unless we can make
  // guarantees about the color space.
  assert_equals(view[0], 94, "Y value at (0, 0)");

  frame.destroy();
}, 'Test we can read planar data from a VideoFrame.');

test(t => {
  let image = makeImageBitmap(32, 16);
  let frame = new VideoFrame(image, {timestamp: 0});

  // TODO(sandersd): This would be more clear as RGBA, but conversion has
  // not be specified (or implemented) yet.
  if (frame.format !== "I420") {
    return;
  }

  assert_equals(frame.planes.length, 3, "number of planes");

  // Attempt to read Y plane data, but destroy the frame first.
  let yPlane = frame.planes[0];
  let yLength = yPlane.length;
  frame.destroy();

  let buffer = new ArrayBuffer(yLength);
  let view = new Uint8Array(buffer);
  assert_throws_dom("InvalidStateError", () => yPlane.readInto(view));
}, 'Test we cannot read planar data from a destroyed VideoFrame.');

test(t => {
  let image = makeImageBitmap(32, 16);

  image.close();

  assert_throws_dom("InvalidStateError", () => {
    let frame = new VideoFrame(image, {timestamp: 10});
  })
}, 'Test constructing VideoFrames from closed ImageBitmap throws.');
