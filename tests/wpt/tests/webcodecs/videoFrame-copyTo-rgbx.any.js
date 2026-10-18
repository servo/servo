// META: global=window,dedicatedworker

// Each pixel has distinct RGB values and a distinct fourth byte, so that any
// byte that is dropped, replaced or moved shows up in the copy.
const kWidth = 4;
const kHeight = 2;
const kPixels = [
  {r: 255, g: 0, b: 0, x: 0x00},
  {r: 0, g: 255, b: 0, x: 0x40},
  {r: 0, g: 0, b: 255, x: 0x7f},
  {r: 255, g: 255, b: 255, x: 0xff},
  {r: 0, g: 0, b: 0, x: 0x01},
  {r: 10, g: 200, b: 90, x: 0x80},
  {r: 200, g: 100, b: 30, x: 0xfe},
  {r: 128, g: 64, b: 192, x: 0x33},
];

function makeData(format) {
  const data = new Uint8Array(kWidth * kHeight * 4);
  kPixels.forEach(({r, g, b, x}, i) => {
    const bgr = format.startsWith('BGR');
    data[4 * i + 0] = bgr ? b : r;
    data[4 * i + 1] = g;
    data[4 * i + 2] = bgr ? r : b;
    data[4 * i + 3] = x;
  });
  return data;
}

function makeFrame(format, data) {
  return new VideoFrame(data, {
    format: format,
    codedWidth: kWidth,
    codedHeight: kHeight,
    timestamp: 0,
  });
}

function validateArrays(original, copy)
{
  assert_equals(original.byteLength, copy.byteLength, "array length");
  for (let i = 0;i < original.byteLength;i++) {
    // FIXME: Revisit the 255 check once https://github.com/w3c/webcodecs/issues/971 is resolved.
    assert_equals(copy[i],  (i % 4) !== 3 ? original[i] : 255);
  }
}

for (const format of ['RGBX', 'BGRX']) {
  promise_test(async t => {
    const data = makeData(format);
    const frame = makeFrame(format, data);
    t.add_cleanup(() => frame.close());
    assert_equals(frame.format, format);

    // Copying without a format option keeps the format of the frame.
    const size = frame.allocationSize();
    assert_equals(size, data.byteLength);
    const copy = new Uint8Array(size);
    const layout = await frame.copyTo(copy);
    assert_equals(layout.length, 1);
    assert_equals(layout[0].stride, kWidth * 4);
    validateArrays(data, copy);
  }, `copyTo() of a ${format} VideoFrame`);

  promise_test(async t => {
    const data = makeData(format);
    const frame = makeFrame(format, data);
    t.add_cleanup(() => frame.close());

    // Explicitly requesting the format of the frame gives the same bytes.
    const options = {format: format};
    const copy = new Uint8Array(frame.allocationSize(options));
    await frame.copyTo(copy, options);
    validateArrays(data, copy);
  }, `copyTo() of a ${format} VideoFrame to ${format}`);

  promise_test(async t => {
    const data = makeData(format);
    const frame = makeFrame(format, data);
    t.add_cleanup(() => frame.close());

    // A copy of the frame keeps the fourth byte as well.
    const clone = new VideoFrame(frame);
    t.add_cleanup(() => clone.close());
    assert_equals(clone.format, format);
    const copy = new Uint8Array(clone.allocationSize());
    await clone.copyTo(copy);
    validateArrays(data, copy);
  }, `copyTo() of a clone of a ${format} VideoFrame`);
}

// An RGBX or BGRX frame is opaque: copying it to a format with alpha must give opaque pixels
for (const [from, to, tolerance] of [
         ['BGRX', 'BGRA', 0], ['RGBX', 'RGBA', 0], ['BGRX', 'RGBA', 1],
         ['RGBX', 'BGRA', 1]]) {
  promise_test(async t => {
    // The padding byte of the pixels is not 0xff for most of them.
    const frame = makeFrame(from, makeData(from));
    t.add_cleanup(() => frame.close());

    const options = {format: to};
    const copy = new Uint8Array(frame.allocationSize(options));
    const layout = await frame.copyTo(copy, options);
    assert_equals(layout.length, 1);

    const expected = makeData(to);
    assert_equals(copy.byteLength, expected.byteLength, 'array length');
    for (let i = 0; i < copy.byteLength; i++) {
      if (i % 4 === 3) {
        assert_equals(copy[i], 255, `alpha of pixel ${(i - 3) / 4}`);
      } else {
        assert_approx_equals(
            copy[i], expected[i], tolerance, `byte ${i} of the copy`);
      }
    }
  }, `copyTo() of a ${from} VideoFrame to ${to} gives opaque pixels`);
}
