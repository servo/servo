"use strict";
var TextDetectionTest = (() => {
  // Class that mocks TextDetection interface defined in
  // https://cs.chromium.org/chromium/src/services/shape_detection/public/mojom/textdetection.mojom
  class MockTextDetection {
    constructor() {
      this.bindingSet_ =
          new mojo.BindingSet(shapeDetection.mojom.TextDetection);

      this.interceptor_ =
          new MojoInterfaceInterceptor(shapeDetection.mojom.TextDetection.name);
      this.interceptor_.oninterfacerequest =
          e => this.bindingSet_.addBinding(this, e.handle);
      this.interceptor_.start();
    }

    detect(bitmapData) {
      this.bufferData_ =
          new Uint32Array(getArrayBufferFromBigBuffer(bitmapData.pixelData));
      return Promise.resolve({
        results: [
          {
            rawValue : "cats",
            boundingBox: { x: 1.0, y: 1.0, width: 100.0, height: 100.0 },
            cornerPoints: [
              { x: 1.0, y: 1.0 },
              { x: 101.0, y: 1.0 },
              { x: 101.0, y: 101.0 },
              { x: 1.0, y: 101.0 }
            ]
          },
          {
            rawValue : "dogs",
            boundingBox: { x: 2.0, y: 2.0, width: 50.0, height: 50.0 },
            cornerPoints: [
              { x: 2.0, y: 2.0 },
              { x: 52.0, y: 2.0 },
              { x: 52.0, y: 52.0 },
              { x: 2.0, y: 52.0 }
            ]
          },
        ],
      });
    }

    getFrameData() {
      return this.bufferData_;
    }

    reset() {
      this.bindingSet_.closeAllBindings();
      this.interceptor_.stop();
    }

  }

  let testInternal = {
    initialized: false,
    MockTextDetection: null
  }

  class TextDetectionTestChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      testInternal.MockTextDetection = new MockTextDetection;
      testInternal.initialized = true;
    }

    // Resets state of text detection mocks between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.MockTextDetection.reset();
      testInternal.MockTextDetection = null;
      testInternal.initialized = false;

      await new Promise(resolve => setTimeout(resolve, 0));
    }

    MockTextDetection() {
      return testInternal.MockTextDetection;
    }
  }

  return TextDetectionTestChromium;

})();
