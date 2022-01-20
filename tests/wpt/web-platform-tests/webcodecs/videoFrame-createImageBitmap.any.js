// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

function testImageBitmapToAndFromVideoFrame(width, height, expectedPixel,
  canvasOptions, imageBitmapOptions, imageSetting) {
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d', canvasOptions);
  ctx.fillStyle = 'rgb(50, 100, 150)';
  ctx.fillRect(0, 0, width, height);
  testCanvas(ctx, width, height, expectedPixel, imageSetting, assert_equals);

  return createImageBitmap(canvas, imageBitmapOptions)
    .then((fromImageBitmap) => {
      let videoFrame = new VideoFrame(fromImageBitmap, {
        timestamp: 0
      });
      return createImageBitmap(videoFrame, imageBitmapOptions);
    })
    .then((toImageBitmap) => {
      let myCanvas = new OffscreenCanvas(width, height);
      let myCtx = myCanvas.getContext('2d', canvasOptions);
      myCtx.drawImage(toImageBitmap, 0, 0);
      let tolerance = 2;
      testCanvas(myCtx, width, height, expectedPixel, imageSetting, (actual, expected) => {
        assert_approx_equals(actual, expected, tolerance);
      });
    });
}

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(48, 36, kSRGBPixel);
}, 'ImageBitmap<->VideoFrame with canvas(48x36 srgb uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(480, 360, kSRGBPixel);
}, 'ImageBitmap<->VideoFrame with canvas(480x360 srgb uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(48, 36, kP3Pixel,
    kCanvasOptionsP3Uint8, {
      colorSpaceConversion: "none"
    }, kImageSettingOptionsP3Uint8);
}, 'ImageBitmap<->VideoFrame with canvas(48x36 display-p3 uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(480, 360, kP3Pixel,
    kCanvasOptionsP3Uint8, {
      colorSpaceConversion: "none"
    }, kImageSettingOptionsP3Uint8);
}, 'ImageBitmap<->VideoFrame with canvas(480x360 display-p3 uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(48, 36, kRec2020Pixel,
    kCanvasOptionsRec2020Uint8, {
      colorSpaceConversion: "none"
    }, kImageSettingOptionsRec2020Uint8);
}, 'ImageBitmap<->VideoFrame with canvas(48x36 rec2020 uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(480, 360, kRec2020Pixel,
    kCanvasOptionsRec2020Uint8, {
      colorSpaceConversion: "none"
    }, kImageSettingOptionsRec2020Uint8);
}, 'ImageBitmap<->VideoFrame with canvas(480x360 rec2020 uint8).');

promise_test(async () => {
  const width = 128;
  const height = 128;
  let vfInit = {format: 'RGBA', timestamp: 0,
                codedWidth: width, codedHeight: height,
                displayWidth: width / 2, displayHeight: height / 2};
  let data = new Uint32Array(vfInit.codedWidth * vfInit.codedHeight);
  data.fill(0xFF966432);  // 'rgb(50, 100, 150)';
  let frame = new VideoFrame(data, vfInit);

  let bitmap = await createImageBitmap(frame);
  frame.close();

  assert_equals(bitmap.width, width / 2);
  assert_equals(bitmap.height, height / 2);
  bitmap.close();
}, 'createImageBitmap uses frame display size');