// Helpers for tests checking that the fourth byte of RGBX and BGRX VideoFrames
// is ignored: the frames must be treated as fully opaque.

const kRgbxWidth = 4;
const kRgbxHeight = 2;

// Expected opaque colors, one per pixel of a 4x2 picture.
const kRgbxColors = [
  [255, 0, 0],
  [0, 255, 0],
  [0, 0, 255],
  [255, 255, 255],
  [0, 0, 0],
  [10, 200, 90],
  [200, 100, 30],
  [128, 64, 192],
];

// Creates a 4x2 frame of `kRgbxColors` in `format` ('RGBX' or 'BGRX'). The
// fourth byte of each pixel is `paddingByte`. If it were used as alpha, a
// value other than 0xff would make the pixels partly or fully transparent.
function makeRgbxFrame(format, paddingByte) {
  const data = new Uint8Array(kRgbxWidth * kRgbxHeight * 4);
  kRgbxColors.forEach(([r, g, b], i) => {
    const bgr = format === 'BGRX';
    data[4 * i + 0] = bgr ? b : r;
    data[4 * i + 1] = g;
    data[4 * i + 2] = bgr ? r : b;
    data[4 * i + 3] = paddingByte;
  });
  return new VideoFrame(data, {
    format: format,
    codedWidth: kRgbxWidth,
    codedHeight: kRgbxHeight,
    timestamp: 0,
  });
}

// Checks that `pixels` (RGBA bytes, row by row) hold `kRgbxColors`, opaque.
function verifyOpaqueRgbxPixels(pixels, description) {
  assert_equals(pixels.length, kRgbxWidth * kRgbxHeight * 4, description);
  kRgbxColors.forEach(([r, g, b], i) => {
    const actual = Array.from(pixels.subarray(4 * i, 4 * i + 4));
    assert_array_equals(
        actual, [r, g, b, 255],
        `${description}: pixel ${i} (x=${i % kRgbxWidth}, ` +
            `y=${Math.floor(i / kRgbxWidth)})`);
  });
}

// Registers a promise_test for each of RGBX/BGRX and several padding bytes.
// `testFunction(frame, description)` verifies a source made from the frame.
function runRgbxTests(sourceName, testFunction) {
  for (const format of ['RGBX', 'BGRX']) {
    for (const padding of [0x00, 0x7f, 0xff]) {
      promise_test(async t => {
        const frame = makeRgbxFrame(format, padding);
        t.add_cleanup(() => frame.close());
        assert_equals(frame.format, format);
        await testFunction(
            frame, `${format}, padding 0x${padding.toString(16)}`);
      }, `${sourceName} of a ${format} VideoFrame with padding byte ` +
         `${padding} is opaque with correct RGB`);
    }
  }
}
