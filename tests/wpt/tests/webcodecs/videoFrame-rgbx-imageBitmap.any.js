// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-rgbx-utils.js

function readPixels(source) {
  const ctx = new OffscreenCanvas(kRgbxWidth, kRgbxHeight).getContext('2d');
  ctx.drawImage(source, 0, 0);
  return ctx.getImageData(0, 0, kRgbxWidth, kRgbxHeight).data;
}

runRgbxTests('createImageBitmap()', async (frame, description) => {
  const bitmap = await createImageBitmap(frame);
  try {
    assert_equals(bitmap.width, kRgbxWidth);
    assert_equals(bitmap.height, kRgbxHeight);
    verifyOpaqueRgbxPixels(readPixels(bitmap), description);
  } finally {
    bitmap.close();
  }
});

// The bitmap is a snapshot: closing the frame must not change it.
promise_test(async t => {
  const frame = makeRgbxFrame('RGBX', 0x00);
  const bitmap = await createImageBitmap(frame);
  t.add_cleanup(() => bitmap.close());
  frame.close();
  verifyOpaqueRgbxPixels(readPixels(bitmap), 'bitmap after closing the frame');
}, 'An ImageBitmap of an RGBX VideoFrame stays opaque after closing the frame');

// Cloned frames are also opaque.
promise_test(async t => {
  const frame = makeRgbxFrame('BGRX', 0x00);
  t.add_cleanup(() => frame.close());
  const clone = frame.clone();
  t.add_cleanup(() => clone.close());
  const bitmap = await createImageBitmap(clone);
  t.add_cleanup(() => bitmap.close());
  verifyOpaqueRgbxPixels(readPixels(bitmap), 'bitmap of a cloned frame');
}, 'An ImageBitmap of a cloned BGRX VideoFrame is opaque');
