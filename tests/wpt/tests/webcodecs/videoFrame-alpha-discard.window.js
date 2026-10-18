// META: script=/webcodecs/videoFrame-alpha-discard-utils.js

async function makeTransparentImage() {
  const canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  canvas.getContext('2d').drawImage(makeTransparentCanvas(), 0, 0);
  const image = new Image();
  image.src = canvas.toDataURL('image/png');
  await image.decode();
  return image;
}

function makeTransparentHTMLCanvas() {
  const canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  canvas.getContext('2d').drawImage(makeTransparentCanvas(), 0, 0);
  return canvas;
}

const kWindowSources = {
  'HTMLImageElement': makeTransparentImage,
  'HTMLCanvasElement': makeTransparentHTMLCanvas,
};

for (const [name, makeSource] of Object.entries(kWindowSources)) {
  async function makeDiscardedFrame(t) {
    const source = await makeSource();
    const frame = new VideoFrame(source, {alpha: 'discard', timestamp: 0});
    t.add_cleanup(() => frame.close());
    return frame;
  }

  promise_test(async t => {
    const frame = await makeDiscardedFrame(t);
    assert_in_array(frame.format, ['RGBX', 'BGRX'], 'format');
    verifyOpaquePicture(drawToNewCanvas(frame), 'new canvas');
  }, `Drawing a VideoFrame made from an ${name} with alpha discard gives opaque pixels`);

  promise_test(async t => {
    const frame = await makeDiscardedFrame(t);
    verifyOpaquePicture(drawToNewCanvas(frame, 'white'), 'white background');
  }, `Drawing a VideoFrame made from an ${name} with alpha discard replaces the canvas background`);

  promise_test(async t => {
    const frame = await makeDiscardedFrame(t);
    const clone = frame.clone();
    t.add_cleanup(() => clone.close());
    verifyOpaquePicture(drawToNewCanvas(clone), 'clone');
  }, `Drawing a clone of a VideoFrame made from an ${name} with alpha discard gives opaque pixels`);

  // Control: with alpha keep, the transparent quadrant stays transparent.
  promise_test(async t => {
    const source = await makeSource();
    const frame = new VideoFrame(source, {alpha: 'keep', timestamp: 0});
    t.add_cleanup(() => frame.close());
    assert_array_equals(getPixel(drawToNewCanvas(frame, 'white'), 24, 24), [255, 255, 255, 255], 'white background shows through');
  }, `Drawing a VideoFrame made from an ${name} with alpha keep preserves its transparency`);
}
