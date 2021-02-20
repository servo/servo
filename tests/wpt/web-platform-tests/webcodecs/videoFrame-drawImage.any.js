// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

function testDrawImageFromVideoFrame(
    width, height, expectedPixel, canvasOptions, imageBitmapOptions,
    imageSetting) {
  let vfInit = {timestamp: 0, codedWidth: width, codedHeight: height};
  let u32_data = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  u32_data.fill(0xFF966432); // 'rgb(50, 100, 150)';
  let argbPlaneData = new Uint8Array(u32_data.buffer);
  let argbPlane = {
    src: argbPlaneData,
    stride: width * 4,
    rows: height
  };
  let frame = new VideoFrame('ABGR', [argbPlane], vfInit);
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d', canvasOptions);
  ctx.drawImage(frame, 0, 0);
  testCanvas(ctx, width, height, expectedPixel, imageSetting, assert_equals);
  frame.close();
}

test(() => {
  return testDrawImageFromVideoFrame(48, 36, kSRGBPixel);
}, 'drawImage(VideoFrame) with canvas(48x36 srgb uint8).');

test(() => {
  return testDrawImageFromVideoFrame(480, 360, kSRGBPixel);
}, 'drawImage(VideoFrame) with canvas(480x360 srgb uint8).');

test(() => {
  return testDrawImageFromVideoFrame(
      48, 36, kP3Pixel, kCanvasOptionsP3Uint8, {colorSpaceConversion: 'none'},
      kImageSettingOptionsP3Uint8);
}, 'drawImage(VideoFrame) with canvas(48x36 display-p3 uint8).');

test(() => {
  return testDrawImageFromVideoFrame(
      480, 360, kP3Pixel, kCanvasOptionsP3Uint8, {colorSpaceConversion: 'none'},
      kImageSettingOptionsP3Uint8);
}, 'drawImage(VideoFrame) with canvas(480x360 display-p3 uint8).');

test(() => {
  return testDrawImageFromVideoFrame(
      48, 36, kRec2020Pixel, kCanvasOptionsRec2020Uint8,
      {colorSpaceConversion: 'none'}, kImageSettingOptionsRec2020Uint8);
}, 'drawImage(VideoFrame) with canvas(48x36 rec2020 uint8).');

test(() => {
  return testDrawImageFromVideoFrame(
      480, 360, kRec2020Pixel, kCanvasOptionsRec2020Uint8,
      {colorSpaceConversion: 'none'}, kImageSettingOptionsRec2020Uint8);
}, 'drawImage(VideoFrame) with canvas(480x360 rec2020 uint8).');
