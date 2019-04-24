// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/shape-detection/resources/shapedetection-helpers.js

// See: https://wicg.github.io/shape-detection-api/

'use strict';

idl_test(
  ['shape-detection-api'],
  ['dom', 'geometry'],
  async idl_array => {
    let faceDetectionTest, barcodeDetectionTest;
    try {
      faceDetectionTest =
          await initialize_detection_tests("FaceDetectionTest");
      barcodeDetectionTest =
          await initialize_detection_tests("BarcodeDetectionTest");
      const img = createTestImage();
      const theImageBitmap = await createImageBitmap(img);

      self.faceDetector = new FaceDetector();
      const faceDetectionResult = await faceDetector.detect(theImageBitmap);
      self.detectedFace = faceDetectionResult[0];

      self.barcodeDetector = new BarcodeDetector();
      const barcodeDetectionResult =
          await barcodeDetector.detect(theImageBitmap);
      self.detectedBarcode = barcodeDetectionResult[0];
    } catch (e) {
      // Surfaced in idlharness.js's test_object below.
    } finally {
      faceDetectionTest.reset();
      barcodeDetectionTest.reset();
    }

    idl_array.add_objects({
      FaceDetector: ['faceDetector'],
      DetectedFace: ['detectedFace'],
      BarcodeDetector: ['barcodeDetector'],
      DetectedBarcode: ['detectedBarcode']
    });
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
