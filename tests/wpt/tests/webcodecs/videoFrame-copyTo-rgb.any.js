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

function rgb2yuv(r, g, b) {
  let y = r * .299000 + g * .587000 + b * .114000
  let u = r * -.168736 + g * -.331264 + b * .500000 + 128
  let v = r * .500000 + g * -.418688 + b * -.081312 + 128

  y = Math.round(y);
  u = Math.round(u);
  v = Math.round(v);
  return {
    y, u, v
  }
}

function makeI420Frames(colorSpace) {
  const kYellow = {r: 0xFF, g: 0xFF, b: 0x00};
  const kRed = {r: 0xFF, g: 0x00, b: 0x00};
  const kBlue = {r: 0x00, g: 0x00, b: 0xFF};
  const kGreen = {r: 0x00, g: 0xFF, b: 0x00};
  const kPink = {r: 0xFF, g: 0x78, b: 0xFF};
  const kMagenta = {r: 0xFF, g: 0x00, b: 0xFF};
  const kBlack = {r: 0x00, g: 0x00, b: 0x00};
  const kWhite = {r: 0xFF, g: 0xFF, b: 0xFF};

  const result = [];
  const init = {format: 'I420', timestamp: 0, codedWidth: 4, codedHeight: 4};
  const colors =
      [kYellow, kRed, kBlue, kGreen, kMagenta, kBlack, kWhite, kPink];
  const data = new Uint8Array(24);
  init.colorSpace = colorSpace;
  for (let color of colors) {
    color = rgb2yuv(color.r, color.g, color.b);
    data.fill(color.y, 0, 16);
    data.fill(color.u, 16, 20);
    data.fill(color.v, 20, 24);
    result.push(new VideoFrame(data, init));
  }
  return result;
}

function makeRGBXFrames(colorSpace) {
  const kYellow = 0xFFFF00;
  const kRed = 0xFF0000;
  const kBlue = 0x0000FF;
  const kGreen = 0x00FF00;
  const kBlack = 0x000000;
  const kWhite = 0xFFFFFF;

  const result = [];
  const init = {format: 'RGBX', timestamp: 0, codedWidth: 4, codedHeight: 4};
  const colors = [kYellow, kRed, kBlue, kGreen, kBlack, kWhite];
  const data = new Uint32Array(16);
  init.colorSpace = colorSpace;
  for (let color of colors) {
    data.fill(color, 0, 16);
    result.push(new VideoFrame(data, init));
  }
  return result;
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
  const tolerance = 1;
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
        promise_test(async t => {
          for (let frame of makeI420Frames(frameColorSpace)) {
            await testFrame(frame, colorSpace, pixelFormat);
            frame.close();
          }
        }, `Convert 4x4 ${frameColorSpaceName} I420 frames to ${pixelFormat} / ${colorSpace}`);
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
        promise_test(async t => {
          for (let frame of makeRGBXFrames(frameColorSpace)) {
            await testFrame(frame, colorSpace, pixelFormat);
            frame.close();
          }
        }, `Convert 4x4 ${frameColorSpaceName} RGBX frames to ${pixelFormat} / ${colorSpace}`);
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
