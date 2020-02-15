"use strict";

var ImageCaptureTest = (() => {
  // Class that mocks ImageCapture interface defined in
  // https://cs.chromium.org/chromium/src/media/capture/mojom/image_capture.mojom
  class MockImageCapture {
    constructor() {
      this.interceptor_ =
          new MojoInterfaceInterceptor(media.mojom.ImageCapture.name);
      this.interceptor_.oninterfacerequest =
        e => this.bindingSet_.addBinding(this, e.handle);
      this.interceptor_.start();

      this.state_ = {
        state: {
          supportedWhiteBalanceModes: [
            media.mojom.MeteringMode.SINGLE_SHOT,
            media.mojom.MeteringMode.CONTINUOUS
          ],
          currentWhiteBalanceMode: media.mojom.MeteringMode.CONTINUOUS,
          supportedExposureModes: [
            media.mojom.MeteringMode.MANUAL,
            media.mojom.MeteringMode.SINGLE_SHOT,
            media.mojom.MeteringMode.CONTINUOUS
          ],
          currentExposureMode: media.mojom.MeteringMode.MANUAL,
          supportedFocusModes: [
            media.mojom.MeteringMode.MANUAL,
            media.mojom.MeteringMode.SINGLE_SHOT
          ],
          currentFocusMode: media.mojom.MeteringMode.MANUAL,
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

          redEyeReduction: media.mojom.RedEyeReduction.CONTROLLABLE,
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
          fillLightMode: [
            media.mojom.FillLightMode.AUTO, media.mojom.FillLightMode.FLASH
          ],
        }
      };
      this.settings_ = null;
      this.bindingSet_ = new mojo.BindingSet(media.mojom.ImageCapture);
    }

    reset() {
      this.bindingSet_.closeAllBindings();
      this.interceptor_.stop();
    }

    getPhotoState(source_id) {
      return Promise.resolve(this.state_);
    }

    setOptions(source_id, settings) {
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

    state() {
      return this.state_.state;
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
