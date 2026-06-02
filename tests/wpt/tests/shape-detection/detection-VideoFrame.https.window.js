'use strict';

function createVideoFrame() {
  const canvas = document.createElement('canvas');
  return new VideoFrame(canvas, {timestamp: 0});
}

promise_test(async (t) => {
  const frame = createVideoFrame();
  const detector = new FaceDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(frame));
}, 'FaceDetector.detect() rejects on a VideoFrame');

promise_test(async (t) => {
  const frame = createVideoFrame();
  const detector = new BarcodeDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(frame));
}, 'BarcodeDetector.detect() rejects on a VideoFrame');

promise_test(async (t) => {
  const frame = createVideoFrame();
  const detector = new TextDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(frame));
}, 'TextDetector.detect() rejects on a VideoFrame');
