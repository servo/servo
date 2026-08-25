/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { SkipTestCase } from '../../common/framework/fixture.js';import { getResourcePath } from '../../common/framework/resources.js';import { keysOf } from '../../common/util/data_tables.js';
import { timeout } from '../../common/util/timeout.js';
import { ErrorWithExtra, assert, raceWithRejectOnTimeout } from '../../common/util/util.js';

import { srgbToDisplayP3 } from '../util/color_space_conversion.js';









// MAINTENANCE_TODO: Uses raw floats as expectation in external_texture related cases has some diffs.
// Remove this conversion utils and uses raw float data as expectation in external_textrue
// related cases when resolve this.
export function convertToUnorm8(expectation) {
  const rgba8Unorm = new Uint8ClampedArray(4);
  rgba8Unorm[0] = Math.round(expectation.R * 255.0);
  rgba8Unorm[1] = Math.round(expectation.G * 255.0);
  rgba8Unorm[2] = Math.round(expectation.B * 255.0);
  rgba8Unorm[3] = Math.round(expectation.A * 255.0);

  return new Uint8Array(rgba8Unorm.buffer);
}

// MAINTENANCE_TODO: Add helper function for BT.601 and BT.709 to remove all magic numbers.
// Expectation values about converting video contents to sRGB color space.
// Source video color space affects expected values.
// The process to calculate these expected pixel values can be found:
// https://github.com/gpuweb/cts/pull/2242#issuecomment-1430382811
// and https://github.com/gpuweb/cts/pull/2242#issuecomment-1463273434
const kBt601PixelValue = {
  srgb: {
    red: { R: 0.972945567233341, G: 0.141794376683341, B: -0.0209589916711088, A: 1.0 },
    green: { R: 0.248234279433399, G: 0.984810378661784, B: -0.0564701319494314, A: 1.0 },
    blue: { R: 0.10159735826538, G: 0.135451122863674, B: 1.00262982899724, A: 1.0 },
    yellow: { R: 0.995470750775951, G: 0.992742114518355, B: -0.0701036235167653, A: 1.0 }
  }
};

const kBt709PixelValue = {
  srgb: {
    red: { R: 1.0, G: 0.0, B: 0.0, A: 1.0 },
    green: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 },
    blue: { R: 0.0, G: 0.0, B: 1.0, A: 1.0 },
    yellow: { R: 1.0, G: 1.0, B: 0.0, A: 1.0 }
  }
};

function makeTable({
  table


})



{
  return Object.fromEntries(
    Object.entries(table).map(([k, row]) => [k, { ...row }])

  );
}

// Video expected pixel value table. Finding expected pixel value
// with video color space and dst color space.
export const kVideoExpectedColors = makeTable({
  table: {
    bt601: {
      'display-p3': {
        yellow: srgbToDisplayP3(kBt601PixelValue.srgb.yellow),
        red: srgbToDisplayP3(kBt601PixelValue.srgb.red),
        blue: srgbToDisplayP3(kBt601PixelValue.srgb.blue),
        green: srgbToDisplayP3(kBt601PixelValue.srgb.green)
      },
      srgb: {
        yellow: kBt601PixelValue.srgb.yellow,
        red: kBt601PixelValue.srgb.red,
        blue: kBt601PixelValue.srgb.blue,
        green: kBt601PixelValue.srgb.green
      }
    },
    bt709: {
      'display-p3': {
        yellow: srgbToDisplayP3(kBt709PixelValue.srgb.yellow),
        red: srgbToDisplayP3(kBt709PixelValue.srgb.red),
        blue: srgbToDisplayP3(kBt709PixelValue.srgb.blue),
        green: srgbToDisplayP3(kBt709PixelValue.srgb.green)
      },
      srgb: {
        yellow: kBt709PixelValue.srgb.yellow,
        red: kBt709PixelValue.srgb.red,
        blue: kBt709PixelValue.srgb.blue,
        green: kBt709PixelValue.srgb.green
      }
    }
  }
});

export const kImageExpectedColors = {
  srgb: {
    red: { R: 1.0, G: 0.0, B: 0.0, A: 1.0 },
    green: { R: 0.0, G: 1.0, B: 0.0, A: 1.0 },
    blue: { R: 0.0, G: 0.0, B: 1.0, A: 1.0 },
    yellow: { R: 1.0, G: 1.0, B: 0.0, A: 1.0 }
  }
};

// MAINTENANCE_TODO: Add BT.2020 video in table.
// Video container and codec defines several transform ops to apply to raw decoded frame to display.
// Our test cases covers 'visible rect' and 'rotation'.
// 'visible rect' is associated with the
// video bitstream and should apply to the raw decoded frames before any transformation.
// 'rotation' is associated with the track or presentation and should transform
// the whole visible rect (e.g. 90-degree rotate makes visible rect of vertical video to horizontal)
// The order to apply these transformations is below:

// [raw decoded frame] ----visible rect clipping ---->[visible frame] ---rotation  ---> present
//      ^                                                                   ^
//      |                                                                   |
// coded size                                                           display size
// The table holds test videos meta infos, including mimeType to check browser compatibility
// video color space, raw frame content layout and the frame displayed layout.
export const kVideoInfo = makeTable({
  table: {
    'four-colors-vp8-bt601.webm': {
      mimeType: 'video/webm; codecs=vp8',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-h264-bt601.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-vp9-bt601.webm': {
      mimeType: 'video/webm; codecs=vp9',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-vp9-bt709.webm': {
      mimeType: 'video/webm; codecs=vp9',
      colorSpace: 'bt709',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    // video coded content has been rotate
    'four-colors-h264-bt601-rotate-90.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'red',
        topRightColor: 'green',
        bottomLeftColor: 'yellow',
        bottomRightColor: 'blue'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-h264-bt601-rotate-180.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'green',
        topRightColor: 'blue',
        bottomLeftColor: 'red',
        bottomRightColor: 'yellow'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-h264-bt601-rotate-270.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'blue',
        topRightColor: 'yellow',
        bottomLeftColor: 'green',
        bottomRightColor: 'red'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-vp9-bt601-rotate-90.mp4': {
      mimeType: 'video/mp4; codecs=vp09.00.10.08',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'red',
        topRightColor: 'green',
        bottomLeftColor: 'yellow',
        bottomRightColor: 'blue'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-vp9-bt601-rotate-180.mp4': {
      mimeType: 'video/mp4; codecs=vp09.00.10.08',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'green',
        topRightColor: 'blue',
        bottomLeftColor: 'red',
        bottomRightColor: 'yellow'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-vp9-bt601-rotate-270.mp4': {
      mimeType: 'video/mp4; codecs=vp09.00.10.08',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'blue',
        topRightColor: 'yellow',
        bottomLeftColor: 'green',
        bottomRightColor: 'red'
      },
      display: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      }
    },
    'four-colors-h264-bt601-hflip.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'red',
        topRightColor: 'yellow',
        bottomLeftColor: 'green',
        bottomRightColor: 'blue'
      }
    },
    'four-colors-h264-bt601-vflip.mp4': {
      mimeType: 'video/mp4; codecs=avc1.4d400c',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'blue',
        topRightColor: 'green',
        bottomLeftColor: 'yellow',
        bottomRightColor: 'red'
      }
    },
    'four-colors-vp9-bt601-hflip.mp4': {
      mimeType: 'video/mp4; codecs=vp09.00.10.08',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'red',
        topRightColor: 'yellow',
        bottomLeftColor: 'green',
        bottomRightColor: 'blue'
      }
    },
    'four-colors-vp9-bt601-vflip.mp4': {
      mimeType: 'video/mp4; codecs=vp09.00.10.08',
      colorSpace: 'bt601',
      coded: {
        topLeftColor: 'yellow',
        topRightColor: 'red',
        bottomLeftColor: 'blue',
        bottomRightColor: 'green'
      },
      display: {
        topLeftColor: 'blue',
        topRightColor: 'green',
        bottomLeftColor: 'yellow',
        bottomRightColor: 'red'
      }
    }
  }
});


export const kVideoNames = keysOf(kVideoInfo);

export const kPredefinedColorSpace = ['display-p3', 'srgb'];

/**
 * Starts playing a video and waits for it to be consumable.
 * Returns a promise which resolves after `callback` (which may be async) completes.
 *
 * @param video An HTML5 Video element.
 * @param callback Function to call when video is ready.
 *
 * Adapted from https://github.com/KhronosGroup/WebGL/blob/main/sdk/tests/js/webgl-test-utils.js
 */
export function startPlayingAndWaitForVideo(
video,
callback)
{
  return raceWithRejectOnTimeout(
    new Promise((resolve, reject) => {
      const callbackAndResolve = () =>
      void (async () => {
        try {
          await callback();
          resolve();
        } catch (ex) {
          reject(ex);
        }
      })();
      if (video.error) {
        reject(
          new ErrorWithExtra('Video.error: ' + video.error.message, () => ({ error: video.error }))
        );
        return;
      }

      video.addEventListener(
        'error',
        (event) =>
        reject(
          new ErrorWithExtra('Video received "error" event, message: ' + event.message, () => ({
            event
          }))
        ),
        true
      );

      if (video.requestVideoFrameCallback) {
        video.requestVideoFrameCallback(() => {
          callbackAndResolve();
        });
      } else {
        // If requestVideoFrameCallback isn't available, check each frame if the video has advanced.
        const timeWatcher = () => {
          if (video.currentTime > 0) {
            callbackAndResolve();
          } else {
            requestAnimationFrame(timeWatcher);
          }
        };
        timeWatcher();
      }

      video.loop = true;
      video.muted = true;
      video.preload = 'auto';
      video.play().catch(reject);
    }),
    2000,
    'Video never became ready'
  );
}

/**
 * Fire a `callback` when the script animation reaches a new frame.
 * Returns a promise which resolves after `callback` (which may be async) completes.
 */
export function waitForNextTask(callback) {
  const { promise, callbackAndResolve } = callbackHelper(callback, 'wait for next task timed out');
  timeout(() => {
    callbackAndResolve();
  }, 0);

  return promise;
}

/**
 * Fire a `callback` when the video reaches a new frame.
 * Returns a promise which resolves after `callback` (which may be async) completes.
 *
 * MAINTENANCE_TODO: Find a way to implement this for browsers without requestVideoFrameCallback as
 * well, similar to the timeWatcher path in startPlayingAndWaitForVideo. If that path is proven to
 * work well, we can consider getting rid of the requestVideoFrameCallback path.
 */
export function waitForNextFrame(
video,
callback)
{
  const { promise, callbackAndResolve } = callbackHelper(callback, 'waitForNextFrame timed out');

  if ('requestVideoFrameCallback' in video) {
    video.requestVideoFrameCallback(() => {
      callbackAndResolve();
    });
  } else {
    throw new SkipTestCase('waitForNextFrame currently requires requestVideoFrameCallback');
  }

  return promise;
}

export async function getVideoFrameFromVideoElement(
test,
video)
{
  test.skipIf(video.captureStream === undefined, 'HTMLVideoElement.captureStream is not supported');

  return raceWithRejectOnTimeout(
    new Promise((resolve) => {
      const videoTrack = video.
      captureStream().
      getVideoTracks()[0];
      const trackProcessor = new MediaStreamTrackProcessor({
        track: videoTrack
      });
      const transformer = new TransformStream({
        transform(videoFrame, _controller) {
          videoTrack.stop();
          test.trackForCleanup(videoFrame);
          resolve(videoFrame);
        },
        flush(controller) {
          controller.terminate();
        }
      });
      const trackGenerator = new MediaStreamTrackGenerator({
        kind: 'video'
      });
      trackProcessor.readable.
      pipeThrough(transformer).
      pipeTo(trackGenerator.writable).
      catch(() => {});
    }),
    2000,
    'Video never became ready'
  );
}

/**
 * Create HTMLVideoElement based on VideoName. Check whether video is playable in current
 * browser environment.
 * Returns a HTMLVideoElement.
 *
 * @param t: GPUTest that requires getting HTMLVideoElement
 * @param videoName: Required video name
 *
 */
export function getVideoElement(t, videoName) {
  if (typeof HTMLVideoElement === 'undefined') {
    t.skip('HTMLVideoElement not available');
  }

  const videoElement = document.createElement('video');
  const videoInfo = kVideoInfo[videoName];

  if (videoElement.canPlayType(videoInfo.mimeType) === '') {
    t.skip('Video codec is not supported');
  }

  const videoUrl = getResourcePath(videoName);
  videoElement.src = videoUrl;

  t.trackForCleanup(videoElement);

  return videoElement;
}

/**
 * Helper for doing something inside of a (possibly async) callback (directly, not in a following
 * microtask), and returning a promise when the callback is done.
 * MAINTENANCE_TODO: Use this in startPlayingAndWaitForVideo (and make sure it works).
 */
function callbackHelper(
callback,
timeoutMessage)
{
  let callbackAndResolve;

  const promiseWithoutTimeout = new Promise((resolve, reject) => {
    callbackAndResolve = () =>
    void (async () => {
      try {
        await callback(); // catches both exceptions and rejections
        resolve();
      } catch (ex) {
        reject(ex);
      }
    })();
  });
  const promise = raceWithRejectOnTimeout(promiseWithoutTimeout, 2000, timeoutMessage);
  return { promise, callbackAndResolve: callbackAndResolve };
}

/**
 * Get a MediaStream from the default webcam via `getUserMedia()`.
 */
async function getStreamFromCamera(
test,
videoTrackConstraints)
{
  test.skipIf(typeof navigator === 'undefined', 'navigator does not exist in this environment');
  test.skipIf(
    typeof navigator.mediaDevices === 'undefined' ||
    typeof navigator.mediaDevices.getUserMedia === 'undefined',
    "Browser doesn't support capture frame from camera."
  );

  const stream = await navigator.mediaDevices.getUserMedia({
    audio: false,
    video: videoTrackConstraints
  });
  test.trackForCleanup({
    close() {
      for (const track of stream.getTracks()) {
        track.stop();
      }
    }
  });
  return stream;
}

/**
 * Chrome on macOS (at least) takes a while before it switches from blank frames
 * to real frames. Wait up to 50 frames for something to show up on the camera.
 */
async function waitForNonBlankFrame({
  getSource,
  waitForNextFrame



}) {
  const cvs = document.createElement('canvas');
  [cvs.width, cvs.height] = [4, 4];
  const ctx = cvs.getContext('2d', { willReadFrequently: true });
  let foundNonBlankFrame = false;
  for (let i = 0; i < 50; ++i) {
    ctx.drawImage(getSource(), 0, 0, cvs.width, cvs.height);
    const pixels = new Uint32Array(ctx.getImageData(0, 0, cvs.width, cvs.height).data.buffer);
    // Look only at RGB, ignore alpha.
    if (pixels.some((p) => (p & 0x00ffffff) !== 0)) {
      foundNonBlankFrame = true;
      break;
    }
    await waitForNextFrame();
  }
  assert(foundNonBlankFrame, 'Failed to get a non-blank video frame');
}

/**
 * Uses MediaStreamTrackProcessor to capture a VideoFrame from the camera.
 * Skips the test if not supported.
 * @param videoTrackConstraints - MediaTrackConstraints (e.g. width/height) to pass to
 *     `getUserMedia()`, or `true` if none.
 */
export async function getVideoFrameFromCamera(
test,
videoTrackConstraints)
{
  test.skipIf(
    typeof MediaStreamTrackProcessor === 'undefined',
    'MediaStreamTrackProcessor not supported'
  );

  const stream = await getStreamFromCamera(test, videoTrackConstraints);
  const tracks = stream.getVideoTracks();
  assert(tracks.length > 0, 'no tracks found');
  const track = tracks[0];

  const trackProcessor = new MediaStreamTrackProcessor({ track });
  const reader = trackProcessor.readable.getReader();

  const waitForNextFrame = async () => {
    const result = await reader.read();
    assert(!result.done, "MediaStreamTrackProcessor: Couldn't get valid frame from stream.");
    return result.value;
  };
  let frame = await waitForNextFrame();
  await waitForNonBlankFrame({
    getSource: () => frame,
    async waitForNextFrame() {
      frame.close();
      frame = await waitForNextFrame();
    }
  });

  test.trackForCleanup(frame);
  return frame;
}

/**
 * Create an HTMLVideoElement from the camera stream. Skips the test if not supported.
 * @param videoTrackConstraints - MediaTrackConstraints (e.g. width/height) to pass to
 *     `getUserMedia()`, or `true` if none.
 * @param paused - whether the video should be paused before returning.
 */
export async function getVideoElementFromCamera(
test,
videoTrackConstraints,
paused)
{
  const stream = await getStreamFromCamera(test, videoTrackConstraints);

  // Main thread
  const video = document.createElement('video');
  video.loop = false;
  video.muted = true;
  video.setAttribute('playsinline', '');
  video.srcObject = stream;
  await new Promise((resolve) => {
    video.onloadedmetadata = resolve;
  });
  await startPlayingAndWaitForVideo(video, () => {});

  await waitForNonBlankFrame({
    getSource: () => video,
    waitForNextFrame: () =>
    new Promise((resolve) =>
    video.requestVideoFrameCallback(() => {
      resolve();
    })
    )
  });

  if (paused) {
    // Pause the video so we get consistent readbacks.
    await new Promise((resolve) => {
      video.onpause = resolve;
      video.pause();
    });
  }

  return video;
}

const kFourColorsInfo = {
  display: {
    topLeftColor: 'yellow',
    topRightColor: 'red',
    bottomLeftColor: 'blue',
    bottomRightColor: 'green'
  }
};

export const kEXIFImageInfo = makeTable({
  table: {
    'four-colors.jpg': kFourColorsInfo,
    'four-colors-rotate-90-cw.jpg': kFourColorsInfo,
    'four-colors-rotate-180-cw.jpg': kFourColorsInfo,
    'four-colors-rotate-270-cw.jpg': kFourColorsInfo
  }
});

export const kImageInfo = makeTable({
  table: {
    'four-colors.jpg': kFourColorsInfo,
    'four-colors.png': kFourColorsInfo,
    'four-colors.bmp': kFourColorsInfo,
    'four-colors.webp': kFourColorsInfo,
    'four-colors.gif': kFourColorsInfo,
    'four-colors.avif': kFourColorsInfo,
    'four-colors.ico': kFourColorsInfo,
    'four-colors.svg': kFourColorsInfo
  }
});


export const kImageNames = keysOf(kImageInfo);


export const kEXIFImageNames = keysOf(kEXIFImageInfo);


export const kObjectTypeFromFiles = [
'ImageBitmap-from-Blob',
'ImageBitmap-from-Image',
'Image'];


/**
 * Load image file(e.g. *.jpg) from ImageBitmap, blob or HTMLImageElement. And
 * convert the result to valid source that GPUCopyExternalImageSource supported.
 */
export async function getSourceFromEXIFImageFile(
test,
exifImageName,
objectTypeFromFile)
{
  const imageUrl = getResourcePath(exifImageName);

  switch (objectTypeFromFile) {
    case 'ImageBitmap-from-Blob':{
        // MAINTENANCE_TODO: resource folder path when using service worker is not correct. Return
        // the correct path to load resource in correct place.
        // The wrong path: /out/webgpu/webworker/web_platform/copyToTexture/resources
        test.skipIf(
          globalThis.constructor.name === 'ServiceWorkerGlobalScope',
          'Try to load image resource from serivce worker but the path is not correct.'
        );
        test.skipIf(
          typeof createImageBitmap === 'undefined',
          'createImageBitmap does not exist in this environment'
        );
        // Load image file through fetch.
        const response = await fetch(imageUrl);
        return createImageBitmap(await response.blob());
      }
    case 'ImageBitmap-from-Image':
    case 'Image':{
        // Skip test if HTMLImageElement is not available, e.g. in worker.
        test.skipIf(
          typeof HTMLImageElement === 'undefined',
          'Try to use HTMLImage do image file decoding but HTMLImageElement not available.'
        );

        // Load image file through HTMLImageElement.
        const image = new Image();
        image.src = imageUrl;
        await raceWithRejectOnTimeout(image.decode(), 5000, 'decode image timeout');
        if (objectTypeFromFile === 'Image') {
          return image;
        }

        return createImageBitmap(image);
      }
  }
}

/**
 * Create HTMLImageElement and load image file and waits for it to be loaded.
 * Returns a promise which resolves after `callback` (which may be async) completes.
 *
 * @param imageName An valid imageName in kkImageInfo table .
 * @param callback Function to call when HTMLImageElement is loaded.
 *
 */
export function loadImageFileAndRun(
test,
imageName,
callback)
{
  return raceWithRejectOnTimeout(
    new Promise((resolve, reject) => {
      const callbackAndResolve = (image) =>
      void (async () => {
        try {
          await callback(image);
          resolve();
        } catch (ex) {
          reject(ex);
        }
      })();
      // Skip test if HTMLImageElement is not available, e.g. in worker.
      test.skipIf(
        typeof HTMLImageElement === 'undefined',
        'Try to use HTMLImage do image file decoding but HTMLImageElement not available.'
      );
      const image = new Image();
      image.src = getResourcePath(imageName);
      image.onload = () => {
        callbackAndResolve(image);
      };
    }),
    2000,
    'Video never became ready'
  );
}