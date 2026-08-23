// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js

// Verify that VideoFrame.copyTo populates every row when converting between
// opaque and alpha surface formats (BGRX<->BGRA, RGBX<->RGBA), including
// when source and destination strides differ.

const SENTINEL = 0xDE;

function fillSolidFrame(format, width, height, pixel) {
  const bpp = 4;
  const data = new Uint8Array(width * height * bpp);
  for (let i = 0; i < data.length; i += bpp) {
    data[i] = pixel[0];
    data[i + 1] = pixel[1];
    data[i + 2] = pixel[2];
    data[i + 3] = pixel[3];
  }
  return new VideoFrame(data, {
    format,
    codedWidth: width,
    codedHeight: height,
    timestamp: 0,
  });
}

async function testOpaqueCopyTo(srcFormat, dstFormat, width, height) {
  const pixel = [0x41, 0x42, 0x43, 0xFF];
  const frame = fillSolidFrame(srcFormat, width, height, pixel);
  const buf = new Uint8Array(frame.allocationSize({format: dstFormat}));
  buf.fill(SENTINEL);
  await frame.copyTo(buf, {format: dstFormat});
  frame.close();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const off = (row * width + col) * 4;
      assert_equals(buf[off], pixel[0], `row ${row} col ${col} byte 0`);
      assert_equals(buf[off + 1], pixel[1], `row ${row} col ${col} byte 1`);
      assert_equals(buf[off + 2], pixel[2], `row ${row} col ${col} byte 2`);
      assert_equals(buf[off + 3], 0xFF, `row ${row} col ${col} alpha`);
    }
  }
}

promise_test(async t => {
  await testOpaqueCopyTo('BGRX', 'BGRA', 8, 4);
}, 'BGRX to BGRA copyTo writes all rows (8x4)');

promise_test(async t => {
  await testOpaqueCopyTo('RGBX', 'RGBA', 8, 4);
}, 'RGBX to RGBA copyTo writes all rows (8x4)');

promise_test(async t => {
  await testOpaqueCopyTo('BGRX', 'BGRA', 100, 100);
}, 'BGRX to BGRA copyTo writes all rows (100x100)');

promise_test(async t => {
  await testOpaqueCopyTo('RGBX', 'RGBA', 100, 100);
}, 'RGBX to RGBA copyTo writes all rows (100x100)');

promise_test(async t => {
  await testOpaqueCopyTo('BGRX', 'BGRA', 90, 50);
}, 'BGRX to BGRA copyTo writes all rows (90x50, non-aligned width)');

promise_test(async t => {
  await testOpaqueCopyTo('BGRX', 'BGRA', 7, 3);
}, 'BGRX to BGRA copyTo writes all rows (7x3, odd dimensions)');

promise_test(async t => {
  await testOpaqueCopyTo('BGRA', 'BGRX', 100, 100);
}, 'BGRA to BGRX copyTo writes all rows (100x100)');

promise_test(async t => {
  await testOpaqueCopyTo('RGBA', 'RGBX', 100, 100);
}, 'RGBA to RGBX copyTo writes all rows (100x100)');

promise_test(async t => {
  const width = 100;
  const height = 100;
  const frame = fillSolidFrame('BGRX', width, height, [0x41, 0x42, 0x43, 0xFF]);
  const buf = new Uint8Array(frame.allocationSize({format: 'BGRA'}));
  buf.fill(SENTINEL);
  await frame.copyTo(buf, {format: 'BGRA'});
  frame.close();
  let sentinelCount = 0;
  for (let i = 0; i < buf.length; i++) {
    if (buf[i] === SENTINEL) sentinelCount++;
  }
  assert_equals(sentinelCount, 0, 'No sentinel bytes should remain after copyTo');
}, 'BGRX to BGRA copyTo overwrites all bytes (sentinel test)');

promise_test(async t => {
  const width = 100;
  const height = 100;
  const pixel = [0x41, 0x42, 0x43, 0xFF];
  const encoderConfig = {codec: 'vp8', width, height, bitrate: 5_000_000};

  await checkEncoderSupport(t, encoderConfig);

  const inputFrame = fillSolidFrame('RGBA', width, height, pixel);
  let decodedFrame = null;
  const decoder = new VideoDecoder({
    output(frame) { decodedFrame = frame; },
    error(e) { t.unreached_func('Decoder error: ' + e)(); },
  });
  let encodedChunk = null;
  let decoderConfig = null;
  const encoder = new VideoEncoder({
    output(chunk, meta) {
      encodedChunk = chunk;
      if (meta.decoderConfig) decoderConfig = meta.decoderConfig;
    },
    error(e) { t.unreached_func('Encoder error: ' + e)(); },
  });

  encoder.configure(encoderConfig);
  encoder.encode(inputFrame);
  inputFrame.close();
  await encoder.flush();
  assert_not_equals(encodedChunk, null, 'Got encoded chunk');
  assert_not_equals(decoderConfig, null, 'Got decoder config');

  decoder.configure(decoderConfig);
  decoder.decode(encodedChunk);
  await decoder.flush();
  assert_not_equals(decodedFrame, null, 'Got decoded frame');

  const nativeFormat = decodedFrame.format;
  if (nativeFormat === 'BGRX' || nativeFormat === 'RGBX') {
    const dstFormat = nativeFormat === 'BGRX' ? 'BGRA' : 'RGBA';
    const allocSize = decodedFrame.allocationSize({format: dstFormat});
    const buf = new Uint8Array(allocSize);
    await decodedFrame.copyTo(buf, {format: dstFormat});

    // The input is a solid color, so after encode/decode + format conversion
    // every output row should match row 0 within VP8 lossy tolerance.
    const stride = width * 4;
    const tolerance = 20;
    let badPixels = 0;
    for (let row = 1; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const ref = col * 4;
        const off = row * stride + col * 4;
        if (Math.abs(buf[off] - buf[ref]) > tolerance ||
            Math.abs(buf[off + 1] - buf[ref + 1]) > tolerance ||
            Math.abs(buf[off + 2] - buf[ref + 2]) > tolerance ||
            Math.abs(buf[off + 3] - buf[ref + 3]) > tolerance) {
          badPixels++;
        }
      }
    }
    assert_equals(badPixels, 0,
        'All rows must match row 0 within tolerance (solid color input)');
  }

  decodedFrame.close();
  encoder.close();
  decoder.close();
}, 'Decoded frame opaque-to-alpha copyTo writes all rows');
