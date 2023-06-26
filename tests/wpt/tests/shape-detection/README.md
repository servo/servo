The `shapedetection-helpers.js` tests require implementations of
the `FaceDetectionTest`, `BarcodeDetectionTest` and `TextDetectionTest`
interfaces, which should emulate platform shape detection backends.

The `FaceDetectionTest` interface is defined as:

```
  class FaceDetectionTest {
    async initialize();  // Sets up the testing environment.
    async reset(); // Frees the resources.
    MockFaceDetectionProvider(); //Returns `MockFaceDetectionProvider` interface.
  };

  class MockFaceDetectionProvider {
    getFrameData(); //Gets frame data of detection result.
    getMaxDetectedFaces(); //Gets value of `maxDetectedFaces` in `FaceDetector` constructor
    getFastMode(); //Gets value of `fastMode` in `FaceDetector` constructor
  };
```

The Chromium implementation of the `FaceDetectionTest` interface is located in
[mock-facedetection.js](../resources/chromium/mock-facedetection.js).

The `BarcodeDetectionTest` interface is defined as:

```
  class BarcodeDetectionTest {
    async initialize();  // Sets up the testing environment.
    async reset(); // Frees the resources.
    MockBarcodeDetectionProvider(); //Returns `MockBarcodeDetectionProvider` interface.
  };

  class MockBarcodeDetectionProvider {
    async enumerateSupportedFormats(); //Enumerates supported formats
    getFrameData(); //Gets frame data of detection result.
    getFormats(); //Gets value of `formats` in `BarcodeDetector` constructor
    simulateNoImplementation(); // simulate a 'no implementation available' case
  };
```

The Chromium implementation of the `BarcodeDetectionTest` interface is located in
[mock-barcodedetection.js](../resources/chromium/mock-barcodedetection.js).

The `TextDetectionTest` interface is defined as:

```
  class TextDetectionTest {
    async initialize();  // Sets up the testing environment.
    async reset(); // Frees the resources.
    MockTextDetection(); //Returns `MockTextDetection` interface.
  };

  class MockTextDetection {
    getFrameData(); //Gets frame data of detection result.
  };
```

The Chromium implementation of the `TextDetectionTest` interface is located in
[mock-textdetection.js](../resources/chromium/mock-textdetection.js).

Other browser vendors should provide their own implementations of
the `FaceDetectionTest`, `BarcodeDetectionTest` and `TextDetectionTest`
interfaces.
