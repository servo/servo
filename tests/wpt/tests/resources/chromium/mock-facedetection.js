import {FaceDetectionReceiver, LandmarkType} from '/gen/services/shape_detection/public/mojom/facedetection.mojom.m.js';
import {FaceDetectionProvider, FaceDetectionProviderReceiver} from '/gen/services/shape_detection/public/mojom/facedetection_provider.mojom.m.js';

self.FaceDetectionTest = (() => {
  // Class that mocks FaceDetectionProvider interface defined in
  // https://cs.chromium.org/chromium/src/services/shape_detection/public/mojom/facedetection_provider.mojom
  class MockFaceDetectionProvider {
    constructor() {
      this.receiver_ = new FaceDetectionProviderReceiver(this);

      this.interceptor_ = new MojoInterfaceInterceptor(
          FaceDetectionProvider.$interfaceName);
      this.interceptor_.oninterfacerequest =
         e => this.receiver_.$.bindHandle(e.handle);
      this.interceptor_.start();
    }

    createFaceDetection(request, options) {
      this.mockService_ = new MockFaceDetection(request, options);
    }

    getFrameData() {
      return this.mockService_.bufferData_;
    }

    getMaxDetectedFaces() {
     return this.mockService_.maxDetectedFaces_;
    }

    getFastMode () {
      return this.mockService_.fastMode_;
    }

    reset() {
      this.mockService_ = null;
      this.receiver_.$.close();
      this.interceptor_.stop();
    }
  }

  // Class that mocks FaceDetection interface defined in
  // https://cs.chromium.org/chromium/src/services/shape_detection/public/mojom/facedetection.mojom
  class MockFaceDetection {
    constructor(request, options) {
      this.maxDetectedFaces_ = options.maxDetectedFaces;
      this.fastMode_ = options.fastMode;
      this.receiver_ = new FaceDetectionReceiver(this);
      this.receiver_.$.bindHandle(request.handle);
    }

    detect(bitmapData) {
      this.bufferData_ =
          new Uint32Array(getArrayBufferFromBigBuffer(bitmapData.pixelData));
      return Promise.resolve({
        results: [
          {
            boundingBox: {x: 1.0, y: 1.0, width: 100.0, height: 100.0},
            landmarks: [{
              type: LandmarkType.EYE,
              locations: [{x: 4.0, y: 5.0}]
            },
            {
              type: LandmarkType.EYE,
              locations: [
                {x: 4.0, y: 5.0}, {x: 5.0, y: 4.0}, {x: 6.0, y: 3.0},
                {x: 7.0, y: 4.0}, {x: 8.0, y: 5.0}, {x: 7.0, y: 6.0},
                {x: 6.0, y: 7.0}, {x: 5.0, y: 6.0}
              ]
            }]
          },
          {
            boundingBox: {x: 2.0, y: 2.0, width: 200.0, height: 200.0},
            landmarks: [{
              type: LandmarkType.NOSE,
              locations: [{x: 100.0, y: 50.0}]
            },
            {
              type: LandmarkType.NOSE,
              locations: [
                {x: 80.0, y: 50.0}, {x: 70.0, y: 60.0}, {x: 60.0, y: 70.0},
                {x: 70.0, y: 60.0}, {x: 80.0, y: 70.0}, {x: 90.0, y: 80.0},
                {x: 100.0, y: 70.0}, {x: 90.0, y: 60.0}, {x: 80.0, y: 50.0}
              ]
            }]
          },
          {
            boundingBox: {x: 3.0, y: 3.0, width: 300.0, height: 300.0},
            landmarks: []
          },
        ]
      });
    }
  }

  let testInternal = {
    initialized: false,
    MockFaceDetectionProvider: null
  }

  class FaceDetectionTestChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      testInternal.MockFaceDetectionProvider = new MockFaceDetectionProvider;
      testInternal.initialized = true;
    }

    // Resets state of face detection mocks between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.MockFaceDetectionProvider.reset();
      testInternal.MockFaceDetectionProvider = null;
      testInternal.initialized = false;

      await new Promise(resolve => setTimeout(resolve, 0));
    }

    MockFaceDetectionProvider() {
      return testInternal.MockFaceDetectionProvider;
    }
  }

  return FaceDetectionTestChromium;
})();
