// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/shape-detection/resources/shapedetection-helpers.js

// See: https://wicg.github.io/shape-detection-api/

'use strict';

idl_test(
  ['shape-detection-api'],
  ['dom', 'geometry'],
  async idl_array => {
    idl_array.add_objects({
      FaceDetector: ['faceDetector'],
      DetectedFace: ['detectedFace'],
      BarcodeDetector: ['barcodeDetector'],
      DetectedBarcode: ['detectedBarcode']
    });

    let faceDetectionTest;
    try {
      faceDetectionTest =
          await initialize_detection_tests("FaceDetectionTest");
      const img = createTestImage();
      const theImageBitmap = await createImageBitmap(img);

      self.faceDetector = new FaceDetector();
      const faceDetectionResult = await faceDetector.detect(theImageBitmap);
      self.detectedFace = faceDetectionResult[0];
    } catch (e) {
      // Surfaced in idlharness.js's test_object.
    } finally {
      faceDetectionTest && faceDetectionTest.reset();
    }

    let barcodeDetectionTest;
    try {
      barcodeDetectionTest =
          await initialize_detection_tests("BarcodeDetectionTest");
      const img = createTestImage();
      const theImageBitmap = await createImageBitmap(img);

      self.barcodeDetector = new BarcodeDetector();
      const barcodeDetectionResult =
          await barcodeDetector.detect(theImageBitmap);
      self.detectedBarcode = barcodeDetectionResult[0];
    } catch (e) {
      // Surface in idlharness.js's test_object.
    } finally {
      barcodeDetectionTest && barcodeDetectionTest.reset();
    }
  }
);

function createTestImage() {
  const image = new OffscreenCanvas(100, 50);
  const imgctx = image.getContext('2d');
  imgctx.fillStyle = "#F00";
  imgctx.fillRect(0, 0, 2, 2);
  imgctx.fillStyle = "#0F0";
  imgctx.fillRect(0, 0, 1, 1);
  return image;
}