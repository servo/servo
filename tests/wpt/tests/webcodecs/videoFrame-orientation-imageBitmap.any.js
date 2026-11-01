// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-orientation-utils.js

runOrientationTests('createImageBitmap()', async (frame, orientation, description) => {
  const {width, height} = orientedSize(orientation);
  const bitmap = await createImageBitmap(frame);
  try {
    assert_equals(bitmap.width, width, 'bitmap width');
    assert_equals(bitmap.height, height, 'bitmap height');
    const ctx = new OffscreenCanvas(width, height).getContext('2d');
    ctx.drawImage(bitmap, 0, 0);
    verifyOrientedPixels(ctx.getImageData(0, 0, width, height).data, orientation, description);
  } finally {
    bitmap.close();
  }
});
