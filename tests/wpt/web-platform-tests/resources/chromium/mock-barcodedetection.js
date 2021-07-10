import {BarcodeDetectionReceiver, BarcodeFormat} from '/gen/services/shape_detection/public/mojom/barcodedetection.mojom.m.js';
import {BarcodeDetectionProvider, BarcodeDetectionProviderReceiver} from '/gen/services/shape_detection/public/mojom/barcodedetection_provider.mojom.m.js';

self.BarcodeDetectionTest = (() => {
  // Class that mocks BarcodeDetectionProvider interface defined in
  // https://cs.chromium.org/chromium/src/services/shape_detection/public/mojom/barcodedetection_provider.mojom
  class MockBarcodeDetectionProvider {
    constructor() {
      this.receiver_ = new BarcodeDetectionProviderReceiver(this);

      this.interceptor_ = new MojoInterfaceInterceptor(
          BarcodeDetectionProvider.$interfaceName);
      this.interceptor_.oninterfacerequest = e => {
        if (this.should_close_pipe_on_request_)
          e.handle.close();
        else
          this.receiver_.$.bindHandle(e.handle);
      }
      this.interceptor_.start();
      this.should_close_pipe_on_request_ = false;
    }

    createBarcodeDetection(request, options) {
      this.mockService_ = new MockBarcodeDetection(request, options);
    }

    enumerateSupportedFormats() {
      return {
        supportedFormats: [
          BarcodeFormat.AZTEC,
          BarcodeFormat.DATA_MATRIX,
          BarcodeFormat.QR_CODE,
        ]
      };
    }

    getFrameData() {
      return this.mockService_.bufferData_;
    }

    getFormats() {
     return this.mockService_.options_.formats;
    }

    reset() {
      this.mockService_ = null;
      this.should_close_pipe_on_request_ = false;
      this.receiver_.$.close();
      this.interceptor_.stop();
    }

    // simulate a 'no implementation available' case
    simulateNoImplementation() {
      this.should_close_pipe_on_request_ = true;
    }
  }

  // Class that mocks BarcodeDetection interface defined in
  // https://cs.chromium.org/chromium/src/services/shape_detection/public/mojom/barcodedetection.mojom
  class MockBarcodeDetection {
    constructor(request, options) {
      this.options_ = options;
      this.receiver_ = new BarcodeDetectionReceiver(this);
      this.receiver_.$.bindHandle(request.handle);
    }

    detect(bitmapData) {
      this.bufferData_ =
          new Uint32Array(getArrayBufferFromBigBuffer(bitmapData.pixelData));
      return {
        results: [
          {
            rawValue : "cats",
            boundingBox: { x: 1.0, y: 1.0, width: 100.0, height: 100.0 },
            format: BarcodeFormat.QR_CODE,
            cornerPoints: [
              { x: 1.0, y: 1.0 },
              { x: 101.0, y: 1.0 },
              { x: 101.0, y: 101.0 },
              { x: 1.0, y: 101.0 }
            ],
          },
          {
            rawValue : "dogs",
            boundingBox: { x: 2.0, y: 2.0, width: 50.0, height: 50.0 },
            format: BarcodeFormat.CODE_128,
            cornerPoints: [
              { x: 2.0, y: 2.0 },
              { x: 52.0, y: 2.0 },
              { x: 52.0, y: 52.0 },
              { x: 2.0, y: 52.0 }
            ],
          },
        ],
      };
    }
  }

  let testInternal = {
    initialized: false,
    MockBarcodeDetectionProvider: null
  }

  class BarcodeDetectionTestChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      testInternal.MockBarcodeDetectionProvider = new MockBarcodeDetectionProvider;
      testInternal.initialized = true;
    }

    // Resets state of barcode detection mocks between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.MockBarcodeDetectionProvider.reset();
      testInternal.MockBarcodeDetectionProvider = null;
      testInternal.initialized = false;

      await new Promise(resolve => setTimeout(resolve, 0));
    }

    MockBarcodeDetectionProvider() {
      return testInternal.MockBarcodeDetectionProvider;
    }
  }

  return BarcodeDetectionTestChromium;
})();

self.BarcodeFormat = BarcodeFormat;
