'use strict';

// This polyfill library implements the WebXR Test API as specified here:
// https://github.com/immersive-web/webxr-test-api

class ChromeXRTest {
  constructor() {
    this.mockVRService_ = new MockVRService(mojo.frameInterfaces);
  }

  simulateDeviceConnection(init_params) {
    return Promise.resolve(this.mockVRService_.addRuntime(init_params));
  }

  simulateDeviceDisconnection(device) {
    this.mockVRService_.removeRuntime(device);
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

  removeRuntime(runtime) {
    // We have no way of distinguishing between devices, so just clear the
    // entire list for now.
    // TODO(http://crbug.com/873409) We also have no way right now to disconnect
    // devices.
    this.runtimes_ = [];
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
          return results[i];
        }
      }

      // If there were no successful results, returns a null session.
      return {session: null};
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

    this.service_ = service;

    this.framesOfReference = {};

    if (fakeDeviceInit.supportsImmersive) {
      this.displayInfo_ = this.getImmersiveDisplayInfo();
    } else {
      this.displayInfo_ = this.getNonImmersiveDisplayInfo();
    }

    if (fakeDeviceInit.supportsEnvironmentIntegration) {
      this.displayInfo_.capabilities.canProvideEnvironmentIntegration = true;
    }
  }

  // Test methods.
  setXRPresentationFrameData(poseMatrix, views) {
    if (poseMatrix == null) {
      this.pose_ = null;
    } else {
      this.setPoseFromMatrix(poseMatrix);
    }

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

      if (changed) {
        this.sessionClient_.onChanged(this.displayInfo_);
      }
    }
  }


  setPoseFromMatrix(poseMatrix) {
    this.pose_ = {
      orientation: null,
      position: null,
      angularVelocity: null,
      linearVelocity: null,
      angularAcceleration: null,
      linearAcceleration: null,
      inputState: null,
      poseIndex: 0
    };

    let pose = this.poseFromMatrix(poseMatrix);
    for (let field in pose) {
      if (this.pose_.hasOwnProperty(field)) {
        this.pose_[field] = pose[field];
      }
    }
  }

  poseFromMatrix(m) {
    let orientation = [];

    let m00 = m[0];
    let m11 = m[5];
    let m22 = m[10];
    // The max( 0, ... ) is just a safeguard against rounding error.
    orientation[3] = Math.sqrt(Math.max(0, 1 + m00 + m11 + m22)) / 2;
    orientation[0] = Math.sqrt(Math.max(0, 1 + m00 - m11 - m22)) / 2;
    orientation[1] = Math.sqrt(Math.max(0, 1 - m00 + m11 - m22)) / 2;
    orientation[2] = Math.sqrt(Math.max(0, 1 - m00 - m11 + m22)) / 2;

    let position = [];
    position[0] = m[12];
    position[1] = m[13];
    position[2] = m[14];

    return {
      orientation, position
    }
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
        offset: [-0.032, 0, 0],
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
        offset: [0.032, 0, 0],
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

    return {
      fieldOfView: {
        upDegrees: toDegrees(upTan),
        downDegrees: toDegrees(downTan),
        leftDegrees: toDegrees(leftTan),
        rightDegrees: toDegrees(rightTan)
      },
      offset: [0, 0, 0],
      renderWidth: 20,
      renderHeight: 20
    };
  }

  // Mojo function implementations.

  // XRFrameDataProvider implementation.
  getFrameData() {
    if (this.pose_) {
      this.pose_.poseIndex++;
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
        projectionMatrix: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
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
        let dataProviderBinding = new mojo.Binding(
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
}

let XRTest = new ChromeXRTest();
