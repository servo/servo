// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-orientation-utils.js

runOrientationTests('drawImage()', (frame, orientation, description) => {
  const {width, height} = orientedSize(orientation);
  const ctx = new OffscreenCanvas(width, height).getContext('2d');
  ctx.drawImage(frame, 0, 0);
  verifyOrientedPixels(ctx.getImageData(0, 0, width, height).data, orientation, description);
});

// The orientation is also applied when scaling to a destination rectangle.
promise_test(async t => {
  const orientation = {rotation: 90};
  const frame = makeOrientedFrame(orientation);
  t.add_cleanup(() => frame.close());
  const ctx = new OffscreenCanvas(4, 8).getContext('2d');
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(frame, 0, 0, 4, 8);
  const pixels = ctx.getImageData(0, 0, 4, 8).data;
  // Each source pixel covers 2x2 pixels, sample their top left pixel.
  const small = new Uint8Array(2 * 4 * 4);
  for (let y = 0; y < 4; ++y) {
    for (let x = 0; x < 2; ++x) {
      const source = 4 * ((2 * y) * 4 + 2 * x);
      small.set(pixels.subarray(source, source + 4), 4 * (y * 2 + x));
    }
  }
  verifyOrientedPixels(small, orientation, 'scaled');
}, 'drawImage() scales a rotated VideoFrame to the destination rectangle');

// A frame wrapping another frame combines their orientations.
promise_test(async t => {
  const base = makeOrientedFrame({rotation: 90});
  t.add_cleanup(() => base.close());
  const frame = new VideoFrame(base, {rotation: 90});
  t.add_cleanup(() => frame.close());
  const orientation = {rotation: 180};
  verifyOrientedFrameProperties(frame, orientation);
  const {width, height} = orientedSize(orientation);
  const ctx = new OffscreenCanvas(width, height).getContext('2d');
  ctx.drawImage(frame, 0, 0);
  verifyOrientedPixels(ctx.getImageData(0, 0, width, height).data, orientation, 'wrapped frame');
}, 'drawImage() of a VideoFrame wrapping a rotated VideoFrame combines the rotations');

// A frame wrapping a flipped frame: the rotation of the init is applied before the flip of the base frame.
for (const rotation of [90, 270]) {
  promise_test(async t => {
    const base = makeOrientedFrame({rotation: 0, flip: true});
    t.add_cleanup(() => base.close());
    const frame = new VideoFrame(base, {rotation});
    t.add_cleanup(() => frame.close());
    // The flipped image is rotated.
    const orientation = {rotation: 360 - rotation, flip: true};
    verifyOrientedFrameProperties(frame, orientation);
    const {width, height} = orientedSize(orientation);
    const ctx = new OffscreenCanvas(width, height).getContext('2d');
    ctx.drawImage(frame, 0, 0);
    verifyOrientedPixels(ctx.getImageData(0, 0, width, height).data, orientation, 'wrapped frame');
  }, `drawImage() of a VideoFrame wrapping a flipped VideoFrame with an init rotation of ${rotation} rotates the flipped image`);

  promise_test(async t => {
    const base = makeOrientedFrame({rotation, flip: false});
    t.add_cleanup(() => base.close());
    const frame = new VideoFrame(base, {flip: true});
    t.add_cleanup(() => frame.close());
    // The rotated image is flipped.
    const orientation = {rotation, flip: true};
    verifyOrientedFrameProperties(frame, orientation);
    const {width, height} = orientedSize(orientation);
    const ctx = new OffscreenCanvas(width, height).getContext('2d');
    ctx.drawImage(frame, 0, 0);
    verifyOrientedPixels(ctx.getImageData(0, 0, width, height).data, orientation, 'wrapped frame');
  }, `drawImage() of a VideoFrame wrapping a VideoFrame rotated by ${rotation} with an init flip flips the rotated image`);
}
