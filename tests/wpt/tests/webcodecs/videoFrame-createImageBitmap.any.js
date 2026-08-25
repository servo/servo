// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(48, 36, kSRGBPixel);
}, 'ImageBitmap<->VideoFrame with canvas(48x36 srgb uint8).');

promise_test(() => {
  return testImageBitmapToAndFromVideoFrame(480, 360, kSRGBPixel);
}, 'ImageBitmap<->VideoFrame with canvas(480x360 srgb uint8).');

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
