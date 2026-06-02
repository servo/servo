// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

test(_ => {
  let width = 48;
  let height = 36;
  let expectedPixel = kSRGBPixel;
  let canvasOptions = undefined;
  let imageSetting = undefined;
  let tolerance = 5;
  let vfInit =
      {format: 'I420P10', timestamp: 0, codedWidth: width, codedHeight: height};
  let data = new Uint16Array(3 * width * height / 2);
  let uOffset = width * height;
  let vOffset = uOffset + width * height / 4;
  // RGB(50, 100, 150) converted to 8-bit YCbCr using BT.709 YUV matrix, then
  // shifted to produce approximate 10-bit YUV colors. It would be more accurate
  // to directly compute 10-bit colors.
  data.fill(96 << 2, 0, uOffset);
  data.fill(155 << 2, uOffset, vOffset);
  data.fill(104 << 2, vOffset);
  let frame = new VideoFrame(data, vfInit);
  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d', canvasOptions);
  ctx.drawImage(frame, 0, 0);
  testCanvas(ctx, width, height, expectedPixel, imageSetting,
             (actual, expected) => {
                 assert_approx_equals(actual, expected, tolerance);
             });
  frame.close();
}, 'drawImage with 10-bit YUV VideoFrame');
