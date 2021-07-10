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

function testCreateImageBitmapFromVideoFrameVP9Decoder() {
  // Prefers hardware decoders by setting video size as large as 720p.
  const width = 1280;
  const height = 720;

  let canvas = new OffscreenCanvas(width, height);
  let ctx = canvas.getContext('2d');
  ctx.fillStyle = 'rgb(50, 100, 150)';
  ctx.fillRect(0, 0, width, height);

  return createImageBitmap(canvas).then((fromImageBitmap) => {
    let videoFrame = new VideoFrame(fromImageBitmap, {
      timestamp: 0
    });
    return new Promise((resolve, reject) => {
      let processVideoFrame = (frame) => {
        createImageBitmap(frame).then((toImageBitmap) => {
          let myCanvas = new OffscreenCanvas(width, height);
          let myCtx = myCanvas.getContext('2d');
          myCtx.drawImage(toImageBitmap, 0, 0);
          let tolerance = 6;
          try {
            testCanvas(myCtx, width, height, kSRGBPixel, null,
              (actual, expected) => {
                assert_approx_equals(actual, expected, tolerance);
              }
            );
          } catch (error) {
            reject(error);
          }
          resolve('Done.');
        });
      };

      const decoderInit = {
        output: processVideoFrame,
        error: (e) => {
          reject(e);
        }
      };

      const encodedVideoConfig = {
        codec: "vp09.00.10.08",
      };

      let decoder = new VideoDecoder(decoderInit);
      decoder.configure(encodedVideoConfig);

      let processVideoChunk = (chunk) => {
        decoder.decode(chunk);
        decoder.flush();
      };

      const encoderInit = {
        output: processVideoChunk,
        error: (e) => {
          reject(e);
        }
      };

      const videoEncoderConfig = {
        codec: "vp09.00.10.08",
        width: width,
        height: height,
        bitrate: 10e6,
        framerate: 30,
      };

      let encoder = new VideoEncoder(encoderInit);
      encoder.configure(videoEncoderConfig);
      encoder.encode(videoFrame, {
        keyFrame: true
      });
      encoder.flush();
    });
  });
}

promise_test(() => {
  return testCreateImageBitmapFromVideoFrameVP9Decoder();
}, 'Create ImageBitmap for a VideoFrame from VP9 decoder.');
