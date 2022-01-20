'use strict';

async function createClosedImageBitmap(t) {
  const img = new Image();
  const imgWatcher = new EventWatcher(t, img, ['load', 'error']);
  img.src = '/images/green-16x16.png';
  await imgWatcher.wait_for('load');
  const imageBitmap = await createImageBitmap(img);
  imageBitmap.close();
  return imageBitmap;
}

promise_test(async (t) => {
  const imageBitmap = await createClosedImageBitmap(t);
  const detector = new FaceDetector();
  try {
    await detector.detect(imageBitmap);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'FaceDetector.detect() rejects on a closed ImageBitmap');

promise_test(async (t) => {
  const imageBitmap = await createClosedImageBitmap(t);
  const detector = new BarcodeDetector();
  try {
    await detector.detect(imageBitmap);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'BarcodeDetector.detect() rejects on a closed ImageBitmap');

promise_test(async (t) => {
  const imageBitmap = await createClosedImageBitmap(t);
  const detector = new TextDetector();
  try {
    await detector.detect(imageBitmap);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'TextDetector.detect() rejects on a closed ImageBitmap');
