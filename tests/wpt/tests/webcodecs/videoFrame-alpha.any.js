// META: global=window,dedicatedworker

function makeRGBACanvas() {
  let canvas = new OffscreenCanvas(32, 32, {alpha: true});
  let ctx = canvas.getContext('2d');

  // Opaque red quadrant.
  ctx.fillStyle = 'rgba(255, 0, 0, 255)';
  ctx.fillRect(0, 0, 16, 16);

  // Opaque blue quadrant.
  ctx.fillStyle = 'rgba(0, 255, 0, 255)';
  ctx.fillRect(16, 0, 16, 16);

  // Opaque green quadrant.
  ctx.fillStyle = 'rgba(0, 0, 255, 255)';
  ctx.fillRect(0, 16, 16, 16);

  // Remaining quadrant should be transparent black.
  return canvas;
}

function getPixel(ctx, x, y) {
  let data = ctx.getImageData(x, y, 1, 1).data;
  return data[0] * 2 ** 24 + data[1] * 2 ** 16 + data[2] * 2 ** 8 + data[3];
}

function verifyPicture(picture) {
  let canvas = new OffscreenCanvas(32, 32, {alpha: true});
  let ctx = canvas.getContext('2d');
  ctx.drawImage(picture, 0, 0);
  assert_equals(getPixel(ctx,  8,  8), 0xFF0000FF);
  assert_equals(getPixel(ctx, 24,  8), 0x00FF00FF);
  assert_equals(getPixel(ctx,  8, 24), 0x0000FFFF);
  assert_equals(getPixel(ctx, 24, 24), 0x00000000);
}

promise_test(async () => {
  let src = makeRGBACanvas();
  let frame = new VideoFrame(src, {alpha: 'keep', timestamp: 0});
  verifyPicture(frame);
  verifyPicture(await createImageBitmap(frame));
}, 'OffscreenCanvas source preserves alpha');

promise_test(async () => {
  let src = makeRGBACanvas().transferToImageBitmap();
  let frame = new VideoFrame(src, {alpha: 'keep', timestamp: 0});
  verifyPicture(frame);
  verifyPicture(await createImageBitmap(frame));
}, 'ImageBitmap source preserves alpha');
