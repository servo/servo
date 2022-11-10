// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

function testDrawImageFromVideoFrame(
    width, height, expectedPixel, canvasOptions, imageSetting) {
  let vfInit =
      {format: 'RGBA', timestamp: 0, codedWidth: width, codedHeight: height};
  let data = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  data.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(data, vfInit);
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d', canvasOptions);
  ctx.drawImage(frame, 0, 0);
  testCanvas(ctx, width, height, expectedPixel, imageSetting, assert_equals);
  frame.close();
}

test(_ => {
  return testDrawImageFromVideoFrame(48, 36, kSRGBPixel);
}, 'drawImage(VideoFrame) with canvas(48x36 srgb uint8).');

test(_ => {
  return testDrawImageFromVideoFrame(480, 360, kSRGBPixel);
}, 'drawImage(VideoFrame) with canvas(480x360 srgb uint8).');

test(_ => {
  return testDrawImageFromVideoFrame(
      48, 36, kP3Pixel, kCanvasOptionsP3Uint8, {colorSpaceConversion: 'none'},
      kImageSettingOptionsP3Uint8);
}, 'drawImage(VideoFrame) with canvas(48x36 display-p3 uint8).');

test(_ => {
  return testDrawImageFromVideoFrame(
      480, 360, kP3Pixel, kCanvasOptionsP3Uint8, {colorSpaceConversion: 'none'},
      kImageSettingOptionsP3Uint8);
}, 'drawImage(VideoFrame) with canvas(480x360 display-p3 uint8).');

test(_ => {
  return testDrawImageFromVideoFrame(
      48, 36, kRec2020Pixel, kCanvasOptionsRec2020Uint8,
      {colorSpaceConversion: 'none'}, kImageSettingOptionsRec2020Uint8);
}, 'drawImage(VideoFrame) with canvas(48x36 rec2020 uint8).');

test(_ => {
  let width = 128;
  let height = 128;
  let vfInit =
      {format: 'RGBA', timestamp: 0, codedWidth: width, codedHeight: height};
  let data = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  data.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(data, vfInit);
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d');

  frame.close();
  assert_throws_dom('InvalidStateError', _ => {
    ctx.drawImage(frame, 0, 0);
  }, 'drawImage with a closed VideoFrame should throw InvalidStateError.');
}, 'drawImage on a closed VideoFrame throws InvalidStateError.');


test(_ => {
  let canvas = new OffscreenCanvas(128, 128);
  let ctx = canvas.getContext('2d');

  let init = {alpha: 'discard', timestamp: 33090};
  let frame = new VideoFrame(canvas, {timestamp: 0});
  let frame2 = new VideoFrame(frame, init);
  let frame3 = new VideoFrame(frame2, init);

  ctx.drawImage(frame3, 0, 0);
  frame.close();
  frame2.close();
  frame3.close();
}, 'drawImage of nested frame works properly');

test(_ => {
  const width = 128;
  const height = 128;
  let vfInit = {format: 'RGBA', timestamp: 0,
                codedWidth: width, codedHeight: height,
                displayWidth: width / 2, displayHeight: height / 2};
  let data = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  data.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(data, vfInit);
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d');
  ctx.fillStyle = "#FFFFFF";
  ctx.fillRect(0, 0, width, height);
  ctx.drawImage(frame, 0, 0);
  frame.close();

  function peekPixel(ctx, x, y) {
    return ctx.getImageData(x, y, 1, 1).data;
  }

  assert_array_equals(peekPixel(ctx, 0, 0), [50, 100, 150, 255]);
  assert_array_equals(peekPixel(ctx, width / 2 - 1, height / 2 - 1),
                                [50, 100, 150, 255]);
  assert_array_equals(peekPixel(ctx, width / 2 + 1, height / 2 + 1),
                                [255, 255, 255, 255]);
  assert_array_equals(peekPixel(ctx, width - 1, height - 1),
                                [255, 255, 255, 255]);
}, 'drawImage with display size != visible size');