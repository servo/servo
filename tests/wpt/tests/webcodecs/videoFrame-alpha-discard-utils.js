// A 32x32 picture with an opaque red, green and blue quadrant and a fully transparent last quadrant.
function makeTransparentCanvas() {
  const canvas = new OffscreenCanvas(32, 32);
  const ctx = canvas.getContext('2d', {alpha: true});
  ctx.fillStyle = 'rgb(255, 0, 0)';
  ctx.fillRect(0, 0, 16, 16);
  ctx.fillStyle = 'rgb(0, 255, 0)';
  ctx.fillRect(16, 0, 16, 16);
  ctx.fillStyle = 'rgb(0, 0, 255)';
  ctx.fillRect(0, 16, 16, 16);
  return canvas;
}

function getPixel(ctx, x, y) {
  return Array.from(ctx.getImageData(x, y, 1, 1).data);
}

// With the alpha discarded, the fully transparent quadrant is opaque black.
function verifyOpaquePicture(ctx, description) {
  assert_array_equals(getPixel(ctx, 8, 8), [255, 0, 0, 255], `${description}: red`);
  assert_array_equals(getPixel(ctx, 24, 8), [0, 255, 0, 255], `${description}: green`);
  assert_array_equals(getPixel(ctx, 8, 24), [0, 0, 255, 255], `${description}: blue`);
  assert_array_equals(getPixel(ctx, 24, 24), [0, 0, 0, 255], `${description}: transparent quadrant`);
}

function drawToNewCanvas(source, background) {
  const ctx = new OffscreenCanvas(32, 32).getContext('2d');
  if (background) {
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, 32, 32);
  }
  ctx.drawImage(source, 0, 0);
  return ctx;
}
