import {BackgroundBlurMode, FillLightMode, ImageCapture, ImageCaptureReceiver, MeteringMode, RedEyeReduction} from '/gen/media/capture/mojom/image_capture.mojom.m.js';

self.ImageCaptureTest = (() => {
  // Class that mocks ImageCapture interface defined in
  // https://cs.chromium.org/chromium/src/media/capture/mojom/image_capture.mojom
  class MockImageCapture {
    constructor() {
      this.interceptor_ =
          new MojoInterfaceInterceptor(ImageCapture.$interfaceName);
      this.interceptor_.oninterfacerequest =
        e => this.receiver_.$.bindHandle(e.handle);
      this.interceptor_.start();

      this.state_ = {
        state: {
          supportedWhiteBalanceModes: [
            MeteringMode.SINGLE_SHOT,
            MeteringMode.CONTINUOUS
          ],
          currentWhiteBalanceMode: MeteringMode.CONTINUOUS,
          supportedExposureModes: [
            MeteringMode.MANUAL,
            MeteringMode.SINGLE_SHOT,
            MeteringMode.CONTINUOUS
          ],
          currentExposureMode: MeteringMode.MANUAL,
          supportedFocusModes: [
            MeteringMode.MANUAL,
            MeteringMode.SINGLE_SHOT
          ],
          currentFocusMode: MeteringMode.MANUAL,
          pointsOfInterest: [{
            x: 0.4,
            y: 0.6
          }],

          exposureCompensation: {
            min: -200.0,
            max: 200.0,
            current: 33.0,
            step: 33.0
          },
          exposureTime: {
            min: 100.0,
            max: 100000.0,
            current: 1000.0,
            step: 100.0
          },
          colorTemperature: {
            min: 2500.0,
            max: 6500.0,
            current: 6000.0,
            step: 1000.0
          },
          iso: {
            min: 100.0,
            max: 12000.0,
            current: 400.0,
            step: 1.0
          },

          brightness: {
            min: 1.0,
            max: 10.0,
            current: 5.0,
            step: 1.0
          },
          contrast: {
            min: 2.0,
            max: 9.0,
            current: 5.0,
            step: 1.0
          },
          saturation: {
            min: 3.0,
            max: 8.0,
            current: 6.0,
            step: 1.0
          },
          sharpness: {
            min: 4.0,
            max: 7.0,
            current: 7.0,
            step: 1.0
          },

          focusDistance: {
            min: 1.0,
            max: 10.0,
            current: 3.0,
            step: 1.0
          },

          pan: {
            min: 0.0,
            max: 10.0,
            current: 5.0,
            step: 2.0
          },

          tilt: {
            min: 0.0,
            max: 10.0,
            current: 5.0,
            step: 2.0
          },

          zoom: {
            min: 0.0,
            max: 10.0,
            current: 5.0,
            step: 5.0
          },

          supportsTorch: true,
          torch: false,

          redEyeReduction: RedEyeReduction.CONTROLLABLE,
          height: {
            min: 240.0,
            max: 2448.0,
            current: 240.0,
            step: 2.0
          },
          width: {
            min: 320.0,
            max: 3264.0,
            current: 320.0,
            step: 3.0
          },
          fillLightMode: [FillLightMode.AUTO, FillLightMode.FLASH],

          supportedBackgroundBlurModes: [
              BackgroundBlurMode.OFF,
              BackgroundBlurMode.BLUR
          ],
          backgroundBlurMode: BackgroundBlurMode.OFF,
        }
      };
      this.panTiltZoomPermissionStatus_ = null;
      this.settings_ = null;
      this.receiver_ = new ImageCaptureReceiver(this);
    }

    reset() {
      this.receiver_.$.close();
      this.interceptor_.stop();
    }

    async getPhotoState(source_id) {
      const shouldKeepPanTiltZoom = await this.isPanTiltZoomPermissionGranted();
      if (shouldKeepPanTiltZoom)
        return Promise.resolve(this.state_);

      const newState = {...this.state_};
      newState.state.pan = {};
      newState.state.tilt = {};
      newState.state.zoom = {};
      return Promise.resolve(newState);
    }

    async setPhotoOptions(source_id, settings) {
      const isAllowedToControlPanTiltZoom = await this.isPanTiltZoomPermissionGranted();
      if (!isAllowedToControlPanTiltZoom &&
          (settings.hasPan || settings.hasTilt || settings.hasZoom)) {
        return Promise.resolve({ success: false });
      }
      this.settings_ = settings;
      if (settings.hasIso)
        this.state_.state.iso.current = settings.iso;
      if (settings.hasHeight)
        this.state_.state.height.current = settings.height;
      if (settings.hasWidth)
        this.state_.state.width.current = settings.width;
      if (settings.hasPan)
        this.state_.state.pan.current = settings.pan;
      if (settings.hasTilt)
        this.state_.state.tilt.current = settings.tilt;
      if (settings.hasZoom)
        this.state_.state.zoom.current = settings.zoom;
      if (settings.hasFocusMode)
        this.state_.state.currentFocusMode = settings.focusMode;
      if (settings.hasFocusDistance)
        this.state_.state.focusDistance.current = settings.focusDistance;

      if (settings.pointsOfInterest.length > 0) {
        this.state_.state.pointsOfInterest =
          settings.pointsOfInterest;
      }

      if (settings.hasExposureMode)
        this.state_.state.currentExposureMode = settings.exposureMode;

      if (settings.hasExposureCompensation) {
        this.state_.state.exposureCompensation.current =
          settings.exposureCompensation;
      }
      if (settings.hasExposureTime) {
        this.state_.state.exposureTime.current =
          settings.exposureTime;
      }
      if (settings.hasWhiteBalanceMode) {
        this.state_.state.currentWhiteBalanceMode =
          settings.whiteBalanceMode;
      }
      if (settings.hasFillLightMode)
        this.state_.state.fillLightMode = [settings.fillLightMode];
      if (settings.hasRedEyeReduction)
        this.state_.state.redEyeReduction = settings.redEyeReduction;
      if (settings.hasColorTemperature) {
        this.state_.state.colorTemperature.current =
          settings.colorTemperature;
      }
      if (settings.hasBrightness)
        this.state_.state.brightness.current = settings.brightness;
      if (settings.hasContrast)
        this.state_.state.contrast.current = settings.contrast;
      if (settings.hasSaturation)
        this.state_.state.saturation.current = settings.saturation;
      if (settings.hasSharpness)
        this.state_.state.sharpness.current = settings.sharpness;

      if (settings.hasTorch)
        this.state_.state.torch = settings.torch;

      if (settings.hasBackgroundBlurMode)
        this.state_.state.backgroundBlurMode = [settings.backgroundBlurMode];

      return Promise.resolve({
        success: true
      });
    }

    takePhoto(source_id) {
      return Promise.resolve({
        blob: {
          mimeType: 'image/cat',
          data: new Array(2)
        }
      });
    }

    async isPanTiltZoomPermissionGranted() {
      if (!this.panTiltZoomPermissionStatus_) {
        this.panTiltZoomPermissionStatus_ = await navigator.permissions.query({
          name: "camera",
          panTiltZoom: true
        });
      }
      return this.panTiltZoomPermissionStatus_.state == "granted";
    }

    state() {
      return this.state_.state;
    }

    turnOffBackgroundBlurMode() {
      this.state_.state.backgroundBlurMode = BackgroundBlurMode.OFF;
    }
    turnOnBackgroundBlurMode() {
      this.state_.state.backgroundBlurMode = BackgroundBlurMode.BLUR;
    }
    turnOffSupportedBackgroundBlurModes() {
      this.state_.state.supportedBackgroundBlurModes = [BackgroundBlurMode.OFF];
    }
    turnOnSupportedBackgroundBlurModes() {
      this.state_.state.supportedBackgroundBlurModes = [BackgroundBlurMode.BLUR];
    }

    options() {
      return this.settings_;
    }
  }

  let testInternal = {
    initialized: false,
    mockImageCapture: null
  }

  class ImageCaptureTestChromium {

    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      testInternal.mockImageCapture = new MockImageCapture;
      testInternal.initialized = true;
    }
    // Resets state of image capture mocks between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.mockImageCapture.reset();
      testInternal.mockImageCapture = null;
      testInternal.initialized = false;

      await new Promise(resolve => setTimeout(resolve, 0));
    }
    mockImageCapture() {
      return testInternal.mockImageCapture;
    }
  }

  return ImageCaptureTestChromium;
})();
