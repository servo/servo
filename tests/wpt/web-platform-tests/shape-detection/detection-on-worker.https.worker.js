importScripts("/resources/testharness.js");
importScripts("resources/shapedetection-helpers.js");

'use strict';

// These tests verify that a Detector's detect() works on an
// ImageBitmap on workers.
const imageBitmapTests =
    [
      {
        createDetector: () => { return new FaceDetector(); },
        mockTestName: "FaceDetectionTest",
        resultSize: 3, // Number of faces
        detectorType: "Face"
      },
      {
        createDetector: () => { return new BarcodeDetector(); },
        mockTestName: "BarcodeDetectionTest",
        resultSize: 2, // Number of barcodes
        detectorType: "Barcode"
      },
      {
        createDetector: () => { return new TextDetector(); },
        mockTestName: "TextDetectionTest",
        resultSize: 2, // Number of text blocks
        detectorType: "Text"
      }
    ];

for (let imageBitmapTest of imageBitmapTests) {
  // ImageBitmap is of transferable type and can be sent to and
  // tested on worker.
  detection_test(imageBitmapTest.mockTestName, async (t, detectionTest) => {
    const img = createTestImage();
    const theImageBitmap = await createImageBitmap(img);
    const detector = imageBitmapTest.createDetector();
    const detectionResult = await detector.detect(theImageBitmap);
    assert_equals(detectionResult.length, imageBitmapTest.resultSize,
      `Number of ${imageBitmapTest.detectorType}`);
  }, `${imageBitmapTest.detectorType} Detector detect(ImageBitmap) on worker`);
}

function createTestImage() {
  const image = new OffscreenCanvas(100, 50);
  const imgctx = image.getContext('2d');
  imgctx.fillStyle = "#F00";
  imgctx.fillRect(0, 0, 2, 2);
  imgctx.fillStyle = "#0F0";
  imgctx.fillRect(0, 0, 1, 1);
  return image;
}

done();
