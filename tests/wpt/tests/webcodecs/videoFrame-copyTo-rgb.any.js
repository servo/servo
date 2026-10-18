// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-utils.js
// META: script=/webcodecs/video-encoder-utils.js

const smpte170m = {
  matrix: 'smpte170m',
  primaries: 'smpte170m',
  transfer: 'smpte170m',
  fullRange: false
};
const bt709 = {
  matrix: 'bt709',
  primaries: 'bt709',
  transfer: 'bt709',
  fullRange: false
};

function compareColors(actual, expected, tolerance, msg) {
  let channel = ['R', 'G', 'B', 'A'];
  for (let i = 0; i < 4; i++) {
    assert_approx_equals(
        actual[i], expected[i], tolerance,
        `${msg} ${channel[i]}: actual: ${actual[i]} expected: ${expected[i]}`);
  }
}

// As per https://w3c.github.io/webcodecs/#videoframe-pick-color-space, we use https://w3c.github.io/webcodecs/#rec709-color-space as default YUV color space.
const kDefaultYUVColorSpace = {matrix: 'bt709', fullRange: false};

// Converts RGB to the YUV values that a frame using `colorSpace` decodes back to the same RGB.
function rgb2yuv(r, g, b, colorSpace) {
  const {matrix, fullRange} = colorSpace ?? kDefaultYUVColorSpace;
  // smpte170m uses the BT.601 matrix.
  const [kr, kb] = matrix === 'bt709' ? [0.2126, 0.0722] : [0.299, 0.114];
  const kg = 1 - kr - kb;

  const yPrime = kr * r + kg * g + kb * b;  // [0, 255]
  const cb = (b - yPrime) / (2 * (1 - kb));  // [-127.5, 127.5]
  const cr = (r - yPrime) / (2 * (1 - kr));  // [-127.5, 127.5]

  const clamp8 = x => Math.max(0, Math.min(255, Math.round(x)));
  if (fullRange) {
    return {y: clamp8(yPrime), u: clamp8(128 + cb), v: clamp8(128 + cr)};
  }

  // Limited range: Y is in [16, 235], U and V are in [16, 240].
  return {
    y: clamp8(16 + yPrime * 219 / 255),
    u: clamp8(128 + cb * 224 / 255),
    v: clamp8(128 + cr * 224 / 255),
  };
}

const kColors = {
  yellow: {r: 0xFF, g: 0xFF, b: 0x00},
  red: {r: 0xFF, g: 0x00, b: 0x00},
  blue: {r: 0x00, g: 0x00, b: 0xFF},
  green: {r: 0x00, g: 0xFF, b: 0x00},
  pink: {r: 0xFF, g: 0x78, b: 0xFF},
  magenta: {r: 0xFF, g: 0x00, b: 0xFF},
  black: {r: 0x00, g: 0x00, b: 0x00},
  white: {r: 0xFF, g: 0xFF, b: 0xFF},
};
const kI420ColorNames =
    ['yellow', 'red', 'blue', 'green', 'magenta', 'black', 'white', 'pink'];
const kRGBXColorNames = ['yellow', 'red', 'blue', 'green', 'black', 'white'];

function makeI420Frame(colorName, colorSpace) {
  const init = {format: 'I420', timestamp: 0, codedWidth: 4, codedHeight: 4};
  if (colorSpace)
    init.colorSpace = colorSpace;
  const color = kColors[colorName];
  const yuv = rgb2yuv(color.r, color.g, color.b, colorSpace);
  const data = new Uint8Array(24);
  data.fill(yuv.y, 0, 16);
  data.fill(yuv.u, 16, 20);
  data.fill(yuv.v, 20, 24);
  return new VideoFrame(data, init);
}

function makeRGBXFrame(colorName, colorSpace) {
  const init = {format: 'RGBX', timestamp: 0, codedWidth: 4, codedHeight: 4};
  if (colorSpace)
    init.colorSpace = colorSpace;
  const color = kColors[colorName];
  // RGBX pixels are R, G, B and a padding byte, which is not alpha.
  const data = new Uint8Array(4 * 4 * 4);
  for (let i = 0; i < data.length; i += 4) {
    data[i] = color.r;
    data[i + 1] = color.g;
    data[i + 2] = color.b;
    data[i + 3] = 0;
  }
  return new VideoFrame(data, init);
}

async function testFrame(frame, colorSpace, pixelFormat) {
  const width = frame.visibleRect.width;
  const height = frame.visibleRect.height;
  let frame_message = 'Frame: ' + JSON.stringify({
    format: frame.format,
    width: width,
    height: height,
    matrix: frame.colorSpace?.matrix,
    primaries: frame.colorSpace?.primaries,
    transfer: frame.colorSpace?.transfer,
  });
  const cnv = new OffscreenCanvas(width, height);
  const ctx =
      cnv.getContext('2d', {colorSpace: colorSpace, willReadFrequently: true});

  // Read VideoFrame pixels via copyTo()
  let imageData = ctx.createImageData(width, height);
  let copy_to_buf = imageData.data.buffer;
  let layout = null;
  try {
    const options = {
      rect: {x: 0, y: 0, width: width, height: height},
      format: pixelFormat,
      colorSpace: colorSpace
    };
    assert_equals(frame.allocationSize(options), copy_to_buf.byteLength);
    layout = await frame.copyTo(copy_to_buf, options);
  } catch (e) {
    assert_unreached(`copyTo() failure: ${e}`);
    return;
  }
  if (layout.length != 1) {
    assert_unreached('Conversion to RGB is not supported by the browser');
    return;
  }

  // Read VideoFrame pixels via drawImage()
  ctx.drawImage(frame, 0, 0, width, height, 0, 0, width, height);
  imageData = ctx.getImageData(0, 0, width, height, {colorSpace: colorSpace});
  let get_image_buf = imageData.data.buffer;

  // Compare!
  let tolerance = 1;
  if (!frame.format.startsWith('BGR') && !frame.format.startsWith('RGB')) {
    // We increase the tolerance in case we have to do YUV -> RGB conversion.
    tolerance++;
  }
  for (let i = 0; i < copy_to_buf.byteLength; i += 4) {
    if (pixelFormat.startsWith('BGR')) {
      // getImageData() always gives us RGB, we need to swap bytes before
      // comparing them with BGR.
      new Uint8Array(get_image_buf, i, 3).reverse();
    }
    compareColors(
        new Uint8Array(copy_to_buf, i, 4), new Uint8Array(get_image_buf, i, 4),
        tolerance, frame_message + ` Mismatch at offset ${i}`);
  }
}

function test_4x4_I420_frames() {
  for (let colorSpace of ['srgb', 'display-p3']) {
    for (let pixelFormat of ['RGBA', 'RGBX', 'BGRA', 'BGRX']) {
      for (let frameColorSpace of [null, smpte170m, bt709]) {
        const frameColorSpaceName = frameColorSpace? frameColorSpace.primaries : "null";
        for (const colorName of kI420ColorNames) {
          promise_test(async t => {
            const frame = makeI420Frame(colorName, frameColorSpace);
            t.add_cleanup(() => frame.close());
            await testFrame(frame, colorSpace, pixelFormat);
          }, `Convert 4x4 ${frameColorSpaceName} ${colorName} I420 frame to ${pixelFormat} / ${colorSpace}`);
        }
      }
    }
  }
}
test_4x4_I420_frames();

function test_4x4_RGB_frames() {
  for (let colorSpace of ['srgb', 'display-p3']) {
    for (let pixelFormat of ['RGBA', 'RGBX', 'BGRA', 'BGRX']) {
      for (let frameColorSpace of [null, smpte170m, bt709]) {
        const frameColorSpaceName = frameColorSpace? frameColorSpace.primaries : "null";
        for (const colorName of kRGBXColorNames) {
          promise_test(async t => {
            const frame = makeRGBXFrame(colorName, frameColorSpace);
            t.add_cleanup(() => frame.close());
            await testFrame(frame, colorSpace, pixelFormat);
          }, `Convert 4x4 ${frameColorSpaceName} ${colorName} RGBX frame to ${pixelFormat} / ${colorSpace}`);
        }
      }
    }
  }
}
test_4x4_RGB_frames();


function test_4color_canvas_frames() {
  for (let colorSpace of ['srgb', 'display-p3']) {
    for (let pixelFormat of ['RGBA', 'RGBX', 'BGRA', 'BGRX']) {
      promise_test(async t => {
        const frame = createFrame(32, 16);
        await testFrame(frame, colorSpace, pixelFormat);
        frame.close();
      }, `Convert 4-color canvas frame to ${pixelFormat} / ${colorSpace}`);
    }
  }
}
test_4color_canvas_frames();

promise_test(async t => {
  let pixelFormat = 'RGBA'
  const init = {format: 'RGBA', timestamp: 0, codedWidth: 4, codedHeight: 4};
  const src_data = new Uint32Array(init.codedWidth * init.codedHeight);
  src_data.fill(0xFFFFFFFF);
  const offset = 5;
  const stride = 40;
  const dst_data = new Uint8Array(offset + stride * init.codedHeight);
  const options = {
    format: pixelFormat,
    layout: [
      {offset: offset, stride: stride},
    ]
  };
  const frame = new VideoFrame(src_data, init);
  await frame.copyTo(dst_data, options)
  assert_false(dst_data.slice(0, offset).some(e => e != 0), 'offset');
  for (let row = 0; row < init.codedHeight; ++row) {
    let width = init.codedWidth * 4;
    const row_data =
        dst_data.slice(offset + stride * row, offset + stride * row + width);
    const margin_data = dst_data.slice(
        offset + stride * row + width, offset + stride * (row + 1));

    assert_false(
        row_data.some(e => e != 0xFF),
        `unexpected data in row ${row} [${row_data}]`);
    assert_false(
        margin_data.some(e => e != 0),
        `unexpected margin in row ${row} [${margin_data}]`);
  }

  frame.close();
}, `copyTo() with layout`);

function test_unsupported_pixel_formats() {
  const kUnsupportedFormats = [
    'I420', 'I420P10', 'I420P12', 'I420A', 'I422', 'I422A', 'I444', 'I444A',
    'NV12'
  ];

  for (let pixelFormat of kUnsupportedFormats) {
    promise_test(async t => {
      const init =
          {format: 'RGBX', timestamp: 0, codedWidth: 4, codedHeight: 4};
      const data = new Uint32Array(16);
      const options = {format: pixelFormat};
      const frame = new VideoFrame(data, init);
      assert_throws_dom(
        'NotSupportedError', () => frame.allocationSize(options));
      await promise_rejects_dom(
          t, 'NotSupportedError', frame.copyTo(data, options))
      frame.close();
    }, `Unsupported format ${pixelFormat}`);
  }
}
test_unsupported_pixel_formats();
