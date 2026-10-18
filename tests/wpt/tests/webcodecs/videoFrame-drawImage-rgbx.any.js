// META: global=window,dedicatedworker

// Expected opaque colors, one per pixel of a 4x2 picture.
const kColors = [
  [255, 0, 0],
  [0, 255, 0],
  [0, 0, 255],
  [255, 255, 255],
  [0, 0, 0],
  [10, 200, 90],
  [200, 100, 30],
  [128, 64, 192],
];
const kWidth = 4;
const kHeight = 2;

function makeData(format, fourthByte) {
  const data = new Uint8Array(kWidth * kHeight * 4);
  kColors.forEach(([r, g, b], i) => {
    const swapped = format.startsWith('BGR');
    data[4 * i + 0] = swapped ? b : r;
    data[4 * i + 1] = g;
    data[4 * i + 2] = swapped ? r : b;
    data[4 * i + 3] = fourthByte;
  });
  return data;
}

function drawAndVerify(frame, description) {
  const canvas = new OffscreenCanvas(kWidth, kHeight);
  const ctx = canvas.getContext('2d');
  ctx.drawImage(frame, 0, 0);
  const pixels = ctx.getImageData(0, 0, kWidth, kHeight).data;
  kColors.forEach(([r, g, b], i) => {
    const actual = Array.from(pixels.subarray(4 * i, 4 * i + 4));
    assert_array_equals(
        actual, [r, g, b, 255],
        `${description}: pixel ${i} (x=${i % kWidth}, y=${Math.floor(i / kWidth)})`);
  });
}

function makeFrame(format, fourthByte) {
  return new VideoFrame(makeData(format, fourthByte), {
    format: format,
    codedWidth: kWidth,
    codedHeight: kHeight,
    timestamp: 0,
  });
}

// The fourth byte of an RGBX/BGRX pixel is not alpha: whatever its value, the drawn pixels must be opaque and carry the right RGB values.
for (const format of ['RGBX', 'BGRX']) {
  for (const padding of [0x00, 0x7f, 0xff]) {
    promise_test(async t => {
      const frame = makeFrame(format, padding);
      t.add_cleanup(() => frame.close());
      assert_equals(frame.format, format);
      drawAndVerify(frame, `${format}, padding 0x${padding.toString(16)}`);
    }, `Drawing a ${format} VideoFrame with padding byte ${padding} to a canvas gives opaque correct RGB`);
  }
}

promise_test(async t => {
  const rgbx = makeFrame('RGBX', 0x00);
  const bgrx = makeFrame('BGRX', 0x00);
  t.add_cleanup(() => { rgbx.close(); bgrx.close(); });

  const readPixels = frame => {
    const ctx = new OffscreenCanvas(kWidth, kHeight).getContext('2d');
    ctx.drawImage(frame, 0, 0);
    return ctx.getImageData(0, 0, kWidth, kHeight).data;
  };
  assert_array_equals(readPixels(rgbx), readPixels(bgrx));
}, 'RGBX and BGRX VideoFrames with the same colors draw the same pixels');
