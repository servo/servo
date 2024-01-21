// META: script=/shape-detection/resources/shapedetection-helpers.js

const imgUint16 = new ImageData(1024, 1024, {storageFormat: 'uint16'});

// These tests verify that a Detector's detect() can process ImageData with
// uint16 storage format.
const imageDataTests = [
  {
    createDetector: () => {
      return new FaceDetector();
    },
    mockTestName: 'FaceDetectionTest',
    name:
        'FaceDetector.detect() can process uint16 storage format ImageData'
  },
  {
    createDetector: () => {
      return new BarcodeDetector();
    },
    mockTestName: 'BarcodeDetectionTest',
    name:
        'BarcodeDetector.detect() can process uint16 storage format ImageData'
  },
  {
    createDetector: () => {
      return new TextDetector();
    },
    mockTestName: 'TextDetectionTest',
    name:
        'TextDetector.detect() can process uint16 storage format ImageData'
  }
];

for (let imageDataTest of imageDataTests) {
  detection_test(imageDataTest.mockTestName, async () => {
    let detector = imageDataTest.createDetector();
    await detector.detect(imgUint16);
  }, imageDataTest.name);
}
