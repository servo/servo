/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { SkipTestCase } from '../../common/framework/fixture.js';
import { getResourcePath } from '../../common/framework/resources.js';
import { makeTable } from '../../common/util/data_tables.js';
import { timeout } from '../../common/util/timeout.js';
import { ErrorWithExtra, raceWithRejectOnTimeout } from '../../common/util/util.js';

export const kVideoInfo = makeTable(['mimeType'], [undefined], {
  // All video names
  'four-colors-vp8-bt601.webm': ['video/webm; codecs=vp8'],
  'four-colors-theora-bt601.ogv': ['video/ogg; codecs=theora'],
  'four-colors-h264-bt601.mp4': ['video/mp4; codecs=avc1.4d400c'],
  'four-colors-vp9-bt601.webm': ['video/webm; codecs=vp9'],
  'four-colors-vp9-bt709.webm': ['video/webm; codecs=vp9'],
  'four-colors-vp9-bt2020.webm': ['video/webm; codecs=vp9'],
  'four-colors-h264-bt601-rotate-90.mp4': ['video/mp4; codecs=avc1.4d400c'],
  'four-colors-h264-bt601-rotate-180.mp4': ['video/mp4; codecs=avc1.4d400c'],
  'four-colors-h264-bt601-rotate-270.mp4': ['video/mp4; codecs=avc1.4d400c'],
});

// Expectation values about converting video contents to sRGB color space.
// Source video color space affects expected values.
// The process to calculate these expected pixel values can be found:
// https://github.com/gpuweb/cts/pull/2242#issuecomment-1430382811
// and https://github.com/gpuweb/cts/pull/2242#issuecomment-1463273434
const kBt601PixelValue = {
  red: new Float32Array([0.972945567233341, 0.141794376683341, -0.0209589916711088, 1.0]),
  green: new Float32Array([0.248234279433399, 0.984810378661784, -0.0564701319494314, 1.0]),
  blue: new Float32Array([0.10159735826538, 0.135451122863674, 1.00262982899724, 1.0]),
  yellow: new Float32Array([0.995470750775951, 0.992742114518355, -0.0774291236205402, 1.0]),
};

function convertToUnorm8(expectation) {
  const unorm8 = new Uint8ClampedArray(expectation.length);

  for (let i = 0; i < expectation.length; ++i) {
    unorm8[i] = Math.round(expectation[i] * 255.0);
  }

  return new Uint8Array(unorm8.buffer);
}

// kVideoExpectations uses unorm8 results
const kBt601Red = convertToUnorm8(kBt601PixelValue.red);
const kBt601Green = convertToUnorm8(kBt601PixelValue.green);
const kBt601Blue = convertToUnorm8(kBt601PixelValue.blue);
const kBt601Yellow = convertToUnorm8(kBt601PixelValue.yellow);

export const kVideoExpectations = [
  {
    videoName: 'four-colors-vp8-bt601.webm',
    _redExpectation: kBt601Red,
    _greenExpectation: kBt601Green,
    _blueExpectation: kBt601Blue,
    _yellowExpectation: kBt601Yellow,
  },
  {
    videoName: 'four-colors-theora-bt601.ogv',
    _redExpectation: kBt601Red,
    _greenExpectation: kBt601Green,
    _blueExpectation: kBt601Blue,
    _yellowExpectation: kBt601Yellow,
  },
  {
    videoName: 'four-colors-h264-bt601.mp4',
    _redExpectation: kBt601Red,
    _greenExpectation: kBt601Green,
    _blueExpectation: kBt601Blue,
    _yellowExpectation: kBt601Yellow,
  },
  {
    videoName: 'four-colors-vp9-bt601.webm',
    _redExpectation: kBt601Red,
    _greenExpectation: kBt601Green,
    _blueExpectation: kBt601Blue,
    _yellowExpectation: kBt601Yellow,
  },
  {
    videoName: 'four-colors-vp9-bt709.webm',
    _redExpectation: new Uint8Array([255, 0, 0, 255]),
    _greenExpectation: new Uint8Array([0, 255, 0, 255]),
    _blueExpectation: new Uint8Array([0, 0, 255, 255]),
    _yellowExpectation: new Uint8Array([255, 255, 0, 255]),
  },
];

export const kVideoRotationExpectations = [
  {
    videoName: 'four-colors-h264-bt601-rotate-90.mp4',
    _topLeftExpectation: kBt601Red,
    _topRightExpectation: kBt601Green,
    _bottomLeftExpectation: kBt601Yellow,
    _bottomRightExpectation: kBt601Blue,
  },
  {
    videoName: 'four-colors-h264-bt601-rotate-180.mp4',
    _topLeftExpectation: kBt601Green,
    _topRightExpectation: kBt601Blue,
    _bottomLeftExpectation: kBt601Red,
    _bottomRightExpectation: kBt601Yellow,
  },
  {
    videoName: 'four-colors-h264-bt601-rotate-270.mp4',
    _topLeftExpectation: kBt601Blue,
    _topRightExpectation: kBt601Yellow,
    _bottomLeftExpectation: kBt601Green,
    _bottomRightExpectation: kBt601Red,
  },
];

/**
 * Starts playing a video and waits for it to be consumable.
 * Returns a promise which resolves after `callback` (which may be async) completes.
 *
 * @param video An HTML5 Video element.
 * @param callback Function to call when video is ready.
 *
 * Adapted from https://github.com/KhronosGroup/WebGL/blob/main/sdk/tests/js/webgl-test-utils.js
 */
export function startPlayingAndWaitForVideo(video, callback) {
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
        event => reject(new ErrorWithExtra('Video received "error" event', () => ({ event }))),
        true
      );

      if ('requestVideoFrameCallback' in video) {
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
export function waitForNextFrame(video, callback) {
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

export async function getVideoFrameFromVideoElement(test, video) {
  if (video.captureStream === undefined) {
    test.skip('HTMLVideoElement.captureStream is not supported');
  }

  return raceWithRejectOnTimeout(
    new Promise((resolve, reject) => {
      const videoTrack = video.captureStream().getVideoTracks()[0];
      const trackProcessor = new MediaStreamTrackProcessor({
        track: videoTrack,
      });
      const transformer = new TransformStream({
        transform(videoFrame, controller) {
          videoTrack.stop();
          resolve(videoFrame);
        },
        flush(controller) {
          controller.terminate();
        },
      });
      const trackGenerator = new MediaStreamTrackGenerator({
        kind: 'video',
      });
      trackProcessor.readable
        .pipeThrough(transformer)
        .pipeTo(trackGenerator.writable)
        .catch(() => {});
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
  const videoElement = document.createElement('video');
  const videoInfo = kVideoInfo[videoName];

  if (videoElement.canPlayType(videoInfo.mimeType) === '') {
    t.skip('Video codec is not supported');
  }

  const videoUrl = getResourcePath(videoName);
  videoElement.src = videoUrl;

  return videoElement;
}

/**
 * Helper for doing something inside of a (possibly async) callback (directly, not in a following
 * microtask), and returning a promise when the callback is done.
 * MAINTENANCE_TODO: Use this in startPlayingAndWaitForVideo (and make sure it works).
 */
function callbackHelper(callback, timeoutMessage) {
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
