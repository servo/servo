'use strict';

promise_test(async (t) => {
  const image = document.createElementNS("http://www.w3.org/2000/svg", "image");
  const detector = new FaceDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(image));
}, 'FaceDetector.detect() rejects on an SVGImageElement');

promise_test(async (t) => {
  const image = document.createElementNS("http://www.w3.org/2000/svg", "image");
  const detector = new BarcodeDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(image));
}, 'BarcodeDetector.detect() rejects on an SVGImageElement');

promise_test(async (t) => {
  const image = document.createElementNS("http://www.w3.org/2000/svg", "image");
  const detector = new TextDetector();
  await promise_rejects_dom(t, 'NotSupportedError', detector.detect(image));
}, 'TextDetector.detect() rejects on an SVGImageElement');
