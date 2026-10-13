// META: script=/webcodecs/videoFrame-utils.js

promise_test(async t => {
  const W = 5;
  const H = 4;

  const canvas = document.createElement('canvas');
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext('2d');

  ctx.fillStyle = '#00ff00';
  ctx.fillRect(0, 0, W, H);

  const frame = new VideoFrame(canvas, {timestamp: 0});
  t.add_cleanup(() => frame.close());

  const format = frame.format;
  assert_true(
      format === 'RGBA' || format === 'RGBX' ||
      format === 'BGRA' || format === 'BGRX',
      `Expected an RGB(A/X) format, got ${format}`);

  const size = frame.allocationSize();
  const buf = new Uint8Array(size);
  await frame.copyTo(buf);

  const bpp = 4;

  for (let row = 0; row < H; row++) {
    for (let x = 0; x < W; x++) {
      const i = (row * W + x) * bpp;
      const pixel = [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]];

      // Green channel must be 255; red and blue must be 0.
      // Alpha may be 255 for RGBA/BGRA or implementation-defined for X formats.
      const rIdx = (format === 'BGRA' || format === 'BGRX') ? 2 : 0;
      const gIdx = 1;
      const bIdx = (format === 'BGRA' || format === 'BGRX') ? 0 : 2;
      const aIdx = 3;

      assert_equals(pixel[rIdx], 0,
          `row ${row} col ${x}: red channel should be 0, got ${pixel[rIdx]}`);
      assert_equals(pixel[gIdx], 255,
          `row ${row} col ${x}: green channel should be 255, got ${pixel[gIdx]}`);
      assert_equals(pixel[bIdx], 0,
          `row ${row} col ${x}: blue channel should be 0, got ${pixel[bIdx]}`);
      if (format === 'RGBA' || format === 'BGRA') {
        assert_equals(pixel[aIdx], 255,
            `row ${row} col ${x}: alpha should be 255, got ${pixel[aIdx]}`);
      }
    }
  }
}, 'copyTo from canvas-backed VideoFrame returns correct pixels for all rows');

promise_test(async t => {
  // Use a different non-aligned width to ensure this isn't width-specific.
  const W = 3;
  const H = 8;

  const canvas = document.createElement('canvas');
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext('2d');

  // Draw a distinct color per row so stride errors are easily detected.
  for (let row = 0; row < H; row++) {
    ctx.fillStyle = `rgb(${row * 30}, ${100 + row * 10}, ${200 - row * 20})`;
    ctx.fillRect(0, row, W, 1);
  }

  const frame = new VideoFrame(canvas, {timestamp: 0});
  t.add_cleanup(() => frame.close());

  const format = frame.format;
  const size = frame.allocationSize();
  const buf = new Uint8Array(size);
  await frame.copyTo(buf);

  const bpp = 4;

  // Read back canvas pixels for ground truth.
  const imageData = ctx.getImageData(0, 0, W, H);
  const canvasPixels = imageData.data;

  for (let row = 0; row < H; row++) {
    for (let x = 0; x < W; x++) {
      const frameIdx = (row * W + x) * bpp;
      const canvasIdx = (row * W + x) * 4;

      const cR = canvasPixels[canvasIdx];
      const cG = canvasPixels[canvasIdx + 1];
      const cB = canvasPixels[canvasIdx + 2];

      let fR, fG, fB;
      if (format === 'BGRA' || format === 'BGRX') {
        fB = buf[frameIdx];
        fG = buf[frameIdx + 1];
        fR = buf[frameIdx + 2];
      } else {
        fR = buf[frameIdx];
        fG = buf[frameIdx + 1];
        fB = buf[frameIdx + 2];
      }

      assert_approx_equals(fR, cR, 1,
          `row ${row} col ${x}: R mismatch`);
      assert_approx_equals(fG, cG, 1,
          `row ${row} col ${x}: G mismatch`);
      assert_approx_equals(fB, cB, 1,
          `row ${row} col ${x}: B mismatch`);
    }
  }
}, 'copyTo from canvas-backed VideoFrame matches canvas pixel data per row');
