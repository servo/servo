'use strict';

// This polyfill library implements the WebXR Test API as specified here:
// https://github.com/immersive-web/webxr-test-api


let default_standing = new gfx.mojom.Transform();
default_standing.matrix = [1, 0, 0, 0,
                           0, 1, 0, 0,
                           0, 0, 1, 0,
                           0, 1.65, 0, 1];
const default_stage_parameters = {
  standingTransform: default_standing,
  sizeX: 0,
  sizeZ: 0,
  bounds: null
};

class ChromeXRTest {
  constructor() {
    this.mockVRService_ = new MockVRService(mojo.frameInterfaces);
  }

  simulateDeviceConnection(init_params) {
    return Promise.resolve(this.mockVRService_.addRuntime(init_params));
  }

  disconnectAllDevices() {
    this.mockVRService_.removeAllRuntimes(device);
    return Promise.resolve();
  }

  simulateUserActivation(callback) {
    return new Promise(resolve => {
      let button = document.createElement('button');
      button.textContent = 'click to continue test';
      button.style.display = 'block';
      button.style.fontSize = '20px';
      button.style.padding = '10px';
      button.onclick = () => {
        resolve(callback());
        document.body.removeChild(button);
      };
      document.body.appendChild(button);
      test_driver.click(button);
    });
  }
}

// Mocking class definitions

// Mock service implements both the VRService and XRDevice mojo interfaces.
class MockVRService {
  constructor() {
    this.bindingSet_ = new mojo.BindingSet(device.mojom.VRService);
    this.runtimes_ = [];

    this.interceptor_ =
        new MojoInterfaceInterceptor(device.mojom.VRService.name);
    this.interceptor_.oninterfacerequest = e =>
        this.bindingSet_.addBinding(this, e.handle);
    this.interceptor_.start();
  }

  // Test methods
  addRuntime(fakeDeviceInit) {
    let runtime = new MockRuntime(fakeDeviceInit, this);
    this.runtimes_.push(runtime);

    if (this.client_) {
      this.client_.onDeviceChanged();
    }

    return runtime;
  }

  removeAllRuntimes() {
    if (this.client_) {
      this.client_.onDeviceChanged();
    }

    this.runtimes_ = [];
  }

  removeRuntime(device) {
    let index = this.runtimes_.indexOf(device);
    if (index >= 0) {
      this.runtimes_.splice(index, 1);
      if (this.client_) {
        console.error("Notifying client");
        this.client_.onDeviceChanged();
      }
    }
  }

  // VRService implementation.
  requestDevice() {
    if (this.runtimes_.length > 0) {
      let devicePtr = new device.mojom.XRDevicePtr();
      new mojo.Binding(
          device.mojom.XRDevice, this, mojo.makeRequest(devicePtr));

      return Promise.resolve({device: devicePtr});
    } else {
      return Promise.resolve({device: null});
    }
  }

  setClient(client) {
    this.client_ = client;
  }

  // XRDevice implementation.
  requestSession(sessionOptions, was_activation) {
    let requests = [];
    // Request a session from all the runtimes.
    for (let i = 0; i < this.runtimes_.length; i++) {
      requests[i] = this.runtimes_[i].requestRuntimeSession(sessionOptions);
    }

    return Promise.all(requests).then((results) => {
      // Find and return the first successful result.
      for (let i = 0; i < results.length; i++) {
        if (results[i].session) {
          return {
            result: {
              session : results[i].session,
              $tag :  0
            }
          };
        }
      }

      // If there were no successful results, returns a null session.
      return {
        result: {
          failureReason : device.mojom.RequestSessionResult.NO_RUNTIME_FOUND,
          $tag :  1
        }
      };
    });
  }

  supportsSession(sessionOptions) {
    let requests = [];
    // Check supports on all the runtimes.
    for (let i = 0; i < this.runtimes_.length; i++) {
      requests[i] = this.runtimes_[i].runtimeSupportsSession(sessionOptions);
    }

    return Promise.all(requests).then((results) => {
      // Find and return the first successful result.
      for (let i = 0; i < results.length; i++) {
        if (results[i].supportsSession) {
          return results[i];
        }
      }

      // If there were no successful results, returns false.
      return {supportsSession: false};
    });
  };
}

// Implements XRFrameDataProvider and XRPresentationProvider. Maintains a mock
// for XRPresentationProvider.
class MockRuntime {
  constructor(fakeDeviceInit, service) {
    this.sessionClient_ = new device.mojom.XRSessionClientPtr();
    this.presentation_provider_ = new MockXRPresentationProvider();

    this.pose_ = null;
    this.next_frame_id_ = 0;
    this.bounds_ = null;
    this.send_pose_reset_ = false;

    this.service_ = service;

    this.framesOfReference = {};

    // Initialize DisplayInfo first to set the defaults, then override with
    // anything from the deviceInit
    if (fakeDeviceInit.supportsImmersive) {
      this.displayInfo_ = this.getImmersiveDisplayInfo();
    } else {
      this.displayInfo_ = this.getNonImmersiveDisplayInfo();
    }

    if (fakeDeviceInit.supportsEnvironmentIntegration) {
      this.displayInfo_.capabilities.canProvideEnvironmentIntegration = true;
    }

    if (fakeDeviceInit.viewerOrigin != null) {
      this.setViewerOrigin(fakeDeviceInit.viewerOrigin);
    }

    if (fakeDeviceInit.localToFloorLevelTransform != null) {
      this.setLocalToFloorLevelTransform(fakeDeviceInit.localToFloorLevelTransform);
    }

    // This appropriately handles if the coordinates are null
    this.setBoundsGeometry(fakeDeviceInit.boundsCoordinates);

    this.setViews(fakeDeviceInit.views);
  }

  // Test API methods.
  disconnect() {
    this.service_.removeRuntime(this);
    this.presentation_provider_.Close();
    if (this.sessionClient_.ptr.isBound()) {
      this.sessionClient_.ptr.reset();
    }

    return Promise.resolve();
  }

  setViews(views) {
    if (views) {
      let changed = false;
      for (let i = 0; i < views.length; i++) {
        if (views[i].eye == 'left') {
          this.displayInfo_.leftEye = this.getEye(views[i]);
          changed = true;
        } else if (views[i].eye == 'right') {
          this.displayInfo_.rightEye = this.getEye(views[i]);
          changed = true;
        }
      }

      if (changed && this.sessionClient_.ptr.isBound()) {
        this.sessionClient_.onChanged(this.displayInfo_);
      }
    }
  }

  setViewerOrigin(origin, emulatedPosition = false) {
    let p = origin.position;
    let q = origin.orientation;
    this.pose_ = {
      orientation: { x: q[0], y: q[1], z: q[2], w: q[3] },
      position: { x: p[0], y: p[1], z: p[2] },
      angularVelocity: null,
      linearVelocity: null,
      angularAcceleration: null,
      linearAcceleration: null,
      inputState: null,
      poseIndex: 0
    };
  }

  clearViewerOrigin() {
    this.pose_ = null;
  }

  simulateVisibilityChange(visibilityState) {
    // TODO(https://crbug.com/982099): Chrome currently does not have a way for
    // devices to bubble up any form of visibilityChange.
  }

  setBoundsGeometry(bounds) {
    if (bounds == null) {
      this.bounds_ = null;
    } else if (bounds.length < 3) {
      throw new Error("Bounds must have a length of at least 3");
    } else {
      this.bounds_ = bounds;
    }

    // We can only set bounds if we have stageParameters set; otherwise, we
    // don't know the transform from local space to bounds space.
    // We'll cache the bounds so that they can be set in the future if the
    // floorLevel transform is set, but we won't update them just yet.
    if (this.displayInfo_.stageParameters) {
      this.displayInfo_.stageParameters.bounds = this.bounds_;

      if (this.sessionClient_.ptr.isBound()) {
        this.sessionClient_.onChanged(this.displayInfo_);
      }
    }
  }

  setLocalToFloorLevelTransform(transform) {
    if (!this.displayInfo_.stageParameters) {
      this.displayInfo_.stageParameters = default_stage_parameters;
      this.displayInfo_.stageParameters.bounds = this.bounds_;
    }

    this.displayInfo_.stageParameters.standingTransform = new gfx.mojom.Transform();
    this.displayInfo_.stageParameters.standingTransform.matrix =
      this.getMatrixFromTransform(transform);

    if (this.sessionClient_.ptr.isBound()) {
      this.sessionClient_.onChanged(this.displayInfo_);
    }
  }

  clearLocalToFloorLevelTransform() {
    if (this.displayInfo_.stageParameters) {
      this.displayInfo_.stageParameters = null;

      if (this.sessionClient_.ptr.isBound()) {
        this.sessionClient_.onChanged(this.displayInfo_);
      }
    }
  }

  simulateResetPose() {
    this.send_pose_reset_ = true;
  }

  // Helper methods
  getMatrixFromTransform(transform) {
    let x = transform.orientation[0];
    let y = transform.orientation[1];
    let z = transform.orientation[2];
    let w = transform.orientation[3];

    let m11 = 1.0 - 2.0 * (y * y + z * z);
    let m21 = 2.0 * (x * y + z * w);
    let m31 = 2.0 * (x * z - y * w);

    let m12 = 2.0 * (x * y - z * w);
    let m22 = 1.0 - 2.0 * (x * x + z * z);
    let m32 = 2.0 * (y * z + x * w);

    let m13 = 2.0 * (x * z + y * w);
    let m23 = 2.0 * (y * z - x * w);
    let m33 = 1.0 - 2.0 * (x * x + y * y);

    let m14 = transform.position[0];
    let m24 = transform.position[1];
    let m34 = transform.position[2];

    // Column-major linearized order is expected.
    return [m11, m21, m31, 0,
            m12, m22, m32, 0,
            m13, m23, m33, 0,
            m14, m24, m34, 1];
  }
  getNonImmersiveDisplayInfo() {
    let displayInfo = this.getImmersiveDisplayInfo();

    displayInfo.capabilities.canPresent = false;
    displayInfo.leftEye = null;
    displayInfo.rightEye = null;

    return displayInfo;
  }

  // Function to generate some valid display information for the device.
  getImmersiveDisplayInfo() {
    return {
      displayName: 'FakeDevice',
      capabilities: {
        hasPosition: false,
        hasExternalDisplay: false,
        canPresent: true,
        maxLayers: 1
      },
      stageParameters: null,
      leftEye: {
        fieldOfView: {
          upDegrees: 48.316,
          downDegrees: 50.099,
          leftDegrees: 50.899,
          rightDegrees: 35.197
        },
        offset: { x: -0.032, y: 0, z: 0 },
        renderWidth: 20,
        renderHeight: 20
      },
      rightEye: {
        fieldOfView: {
          upDegrees: 48.316,
          downDegrees: 50.099,
          leftDegrees: 50.899,
          rightDegrees: 35.197
        },
        offset: { x: 0.032, y: 0, z: 0 },
        renderWidth: 20,
        renderHeight: 20
      },
      webxrDefaultFramebufferScale: 0.7,
    };
  }

  // This function converts between the matrix provided by the WebXR test API
  // and the internal data representation.
  getEye(fakeXRViewInit) {
    let m = fakeXRViewInit.projectionMatrix;

    function toDegrees(tan) {
      return Math.atan(tan) * 180 / Math.PI;
    }

    let xScale = m[0];
    let yScale = m[5];
    let near = m[14] / (m[10] - 1);
    let far = m[14] / (m[10] - 1);
    let leftTan = (1 - m[8]) / m[0];
    let rightTan = (1 + m[8]) / m[0];
    let upTan = (1 + m[9]) / m[5];
    let downTan = (1 - m[9]) / m[5];

    let offset = fakeXRViewInit.viewOffset.position;

    return {
      fieldOfView: {
        upDegrees: toDegrees(upTan),
        downDegrees: toDegrees(downTan),
        leftDegrees: toDegrees(leftTan),
        rightDegrees: toDegrees(rightTan)
      },
      offset: { x: offset[0], y: offset[1], z: offset[2] },
      renderWidth: fakeXRViewInit.resolution.width,
      renderHeight: fakeXRViewInit.resolution.height
    };
  }

  // Mojo function implementations.

  // XRFrameDataProvider implementation.
  getFrameData() {
    if (this.pose_) {
      this.pose_.poseIndex++;
      this.pose_.poseReset = this.send_pose_reset_;
      this.send_pose_reset_ = false;
    }

    // Convert current document time to monotonic time.
    let now = window.performance.now() / 1000.0;
    let diff = now - internals.monotonicTimeToZeroBasedDocumentTime(now);
    now += diff;
    now *= 1000000;

    return Promise.resolve({
      frameData: {
        pose: this.pose_,
        timeDelta: {
          microseconds: now,
        },
        frameId: this.next_frame_id_++,
        bufferHolder: null,
        bufferSize: {}
      }
    });
  }

  getEnvironmentIntegrationProvider(environmentProviderRequest) {
    this.environmentProviderBinding_ = new mojo.AssociatedBinding(
        device.mojom.XREnvironmentIntegrationProvider, this,
        environmentProviderRequest);
  }

  // Note that if getEnvironmentProvider hasn't finished running yet this will
  // be undefined. It's recommended that you allow a successful task to post
  // first before attempting to close.
  closeEnvironmentIntegrationProvider() {
    this.environmentProviderBinding_.close();
  }

  closeDataProvider() {
    this.dataProviderBinding_.close();
  }

  updateSessionGeometry(frame_size, display_rotation) {
    // This function must exist to ensure that calls to it do not crash, but we
    // do not have any use for this data at present.
  }

  // Utility function
  requestRuntimeSession(sessionOptions) {
    return this.runtimeSupportsSession(sessionOptions).then((result) => {
      // The JavaScript bindings convert c_style_names to camelCase names.
      let options = new device.mojom.XRPresentationTransportOptions();
      options.transportMethod =
          device.mojom.XRPresentationTransportMethod.SUBMIT_AS_MAILBOX_HOLDER;
      options.waitForTransferNotification = true;
      options.waitForRenderNotification = true;

      let submit_frame_sink;
      if (result.supportsSession) {
        submit_frame_sink = {
          clientRequest: this.presentation_provider_.getClientRequest(),
          provider: this.presentation_provider_.bindProvider(sessionOptions),
          transportOptions: options
        };

        let dataProviderPtr = new device.mojom.XRFrameDataProviderPtr();
        let dataProviderRequest = mojo.makeRequest(dataProviderPtr);
        this.dataProviderBinding_ = new mojo.Binding(
            device.mojom.XRFrameDataProvider, this, dataProviderRequest);

        let clientRequest = mojo.makeRequest(this.sessionClient_);

        return Promise.resolve({
          session: {
            submitFrameSink: submit_frame_sink,
            dataProvider: dataProviderPtr,
            clientRequest: clientRequest,
            displayInfo: this.displayInfo_
          }
        });
      } else {
        return Promise.resolve({session: null});
      }
    });
  }

  runtimeSupportsSession(options) {
    return Promise.resolve({
      supportsSession:
          !options.immersive || this.displayInfo_.capabilities.canPresent
    });
  };
}

// Mojo helper classes
class MockXRPresentationProvider {
  constructor() {
    this.binding_ = new mojo.Binding(device.mojom.XRPresentationProvider, this);

    this.submit_frame_count_ = 0;
    this.missing_frame_count_ = 0;
  }

  bindProvider(request) {
    let providerPtr = new device.mojom.XRPresentationProviderPtr();
    let providerRequest = mojo.makeRequest(providerPtr);

    this.binding_.close();

    this.binding_ = new mojo.Binding(
        device.mojom.XRPresentationProvider, this, providerRequest);

    return providerPtr;
  }

  getClientRequest() {
    this.submitFrameClient_ = new device.mojom.XRPresentationClientPtr();
    return mojo.makeRequest(this.submitFrameClient_);
  }

  // XRPresentationProvider mojo implementation
  submitFrameMissing(frameId, mailboxHolder, timeWaited) {
    this.missing_frame_count_++;
  }

  submitFrame(frameId, mailboxHolder, timeWaited) {
    this.submit_frame_count_++;

    // Trigger the submit completion callbacks here. WARNING: The
    // Javascript-based mojo mocks are *not* re-entrant. It's OK to
    // wait for these notifications on the next frame, but waiting
    // within the current frame would never finish since the incoming
    // calls would be queued until the current execution context finishes.
    this.submitFrameClient_.onSubmitFrameTransferred(true);
    this.submitFrameClient_.onSubmitFrameRendered();
  }

  // Utility methods
  Close() {
    this.binding_.close();
  }
}

// This is a temporary workaround for the fact that spinning up webxr before
// the mojo interceptors are created will cause the interceptors to not get
// registered, so we have to create this before we query xr;
let XRTest = new ChromeXRTest();

// This test API is also used to run Chrome's internal legacy VR tests; however,
// those fail if navigator.xr has been used. Those tests will set a bool telling
// us not to try to check navigator.xr
if ((typeof legacy_vr_test === 'undefined') || !legacy_vr_test) {
  // Some tests may run in the http context where navigator.xr isn't exposed
  // This should just be to test that it isn't exposed, but don't try to set up
  // the test framework in this case.
  if (navigator.xr) {
    navigator.xr.test = XRTest;
  }
} else {
  navigator.vr = { test: XRTest };
}
