// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-alpha-discard-utils.js

const kSources = {
  'OffscreenCanvas': () => makeTransparentCanvas(),
  'ImageBitmap': () => makeTransparentCanvas().transferToImageBitmap(),
  'VideoFrame': () => new VideoFrame(
      makeTransparentCanvas(), {alpha: 'keep', timestamp: 0}),
};

for (const [name, makeSource] of Object.entries(kSources)) {
  function makeDiscardedFrame(t) {
    const source = makeSource();
    const frame = new VideoFrame(source, {alpha: 'discard', timestamp: 0});
    t.add_cleanup(() => frame.close());
    if (source.close)
      source.close();
    return frame;
  }

  promise_test(async t => {
    const frame = makeDiscardedFrame(t);
    assert_in_array(frame.format, ['RGBX', 'BGRX'], 'format');
    verifyOpaquePicture(drawToNewCanvas(frame), 'new canvas');
  }, `Drawing a VideoFrame made from an ${name} with alpha discard gives opaque pixels`);

  promise_test(async t => {
    const frame = makeDiscardedFrame(t);
    verifyOpaquePicture(drawToNewCanvas(frame, 'white'), 'white background');
  }, `Drawing a VideoFrame made from an ${name} with alpha discard replaces the canvas background`);

  promise_test(async t => {
    const frame = makeDiscardedFrame(t);
    const clone = frame.clone();
    t.add_cleanup(() => clone.close());
    verifyOpaquePicture(drawToNewCanvas(clone), 'clone');
  }, `Drawing a clone of a VideoFrame made from an ${name} with alpha discard gives opaque pixels`);

  promise_test(async t => {
    const frame = makeDiscardedFrame(t);
    const bitmap = await createImageBitmap(frame);
    t.add_cleanup(() => bitmap.close());
    verifyOpaquePicture(drawToNewCanvas(bitmap), 'ImageBitmap');
  }, `An ImageBitmap of a VideoFrame made from an ${name} with alpha discard is opaque`);
}

promise_test(async t => {
  const kept = new VideoFrame(makeTransparentCanvas(), {alpha: 'keep', timestamp: 0});
  t.add_cleanup(() => kept.close());
  const discarded = new VideoFrame(kept, {alpha: 'discard'});
  t.add_cleanup(() => discarded.close());
  assert_in_array(discarded.format, ['RGBX', 'BGRX'], 'format');
  verifyOpaquePicture(drawToNewCanvas(discarded), 'discarded copy');
  // The original frame is unchanged.
  assert_in_array(kept.format, ['RGBA', 'BGRA'], 'original format');
  assert_array_equals(getPixel(drawToNewCanvas(kept), 24, 24), [0, 0, 0, 0], 'original keeps its transparency');
}, 'Discarding the alpha of a VideoFrame does not change the original frame');

// Control: with alpha keep, the transparent quadrant stays transparent.
promise_test(async t => {
  const frame = new VideoFrame(makeTransparentCanvas(), {alpha: 'keep', timestamp: 0});
  t.add_cleanup(() => frame.close());
  const ctx = drawToNewCanvas(frame, 'white');
  assert_array_equals(getPixel(ctx, 24, 24), [255, 255, 255, 255]);
  assert_array_equals(getPixel(drawToNewCanvas(frame), 24, 24), [0, 0, 0, 0]);
}, 'Drawing a VideoFrame with alpha keep preserves its transparency');
