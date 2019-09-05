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

function getMatrixFromTransform(transform) {
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

function getGFXTransformFromPosition(x, y, z) {
  let transform = new gfx.mojom.Transform();
  transform.matrix = getMatrixFromTransform({
    position: [x, y, z],
    orientation: [0, 0, 0, 1]
  });
  return transform;
}

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
    let button = document.createElement('button');
    button.textContent = 'click to continue test';
    button.style.display = 'block';
    button.style.fontSize = '20px';
    button.style.padding = '10px';
    button.onclick = () => {
      callback();
      document.body.removeChild(button);
    };
    document.body.appendChild(button);
    test_driver.click(button);
  }
}

// Mocking class definitions

// Mock service implements the VRService mojo interface.
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
        this.client_.onDeviceChanged();
      }
    }
  }

  setClient(client) {
    if (this.client_) {
      throw new Error("setClient should only be called once");
    }

    this.client_ = client;
  }

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
          failureReason : device.mojom.RequestSessionError.NO_RUNTIME_FOUND,
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

    this.input_sources_ = [];
    this.next_input_source_index_ = 1;

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

    if (fakeDeviceInit.floorOrigin != null) {
      this.setFloorOrigin(fakeDeviceInit.floorOrigin);
    }

    // This appropriately handles if the coordinates are null
    this.setBoundsGeometry(fakeDeviceInit.boundsCoordinates);

    this.setViews(fakeDeviceInit.views);

    // Need to support webVR which doesn't have a notion of features
    this.setFeatures(fakeDeviceInit.supportedFeatures || []);
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

  setFloorOrigin(floorOrigin) {
    if (!this.displayInfo_.stageParameters) {
      this.displayInfo_.stageParameters = default_stage_parameters;
      this.displayInfo_.stageParameters.bounds = this.bounds_;
    }

    this.displayInfo_.stageParameters.standingTransform = new gfx.mojom.Transform();
    this.displayInfo_.stageParameters.standingTransform.matrix =
      getMatrixFromTransform(floorOrigin);

    if (this.sessionClient_.ptr.isBound()) {
      this.sessionClient_.onChanged(this.displayInfo_);
    }
  }

  clearFloorOrigin() {
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

  simulateInputSourceConnection(fakeInputSourceInit) {
    let index = this.next_input_source_index_;
    this.next_input_source_index_++;

    let source = new MockXRInputSource(fakeInputSourceInit, index, this);
    this.input_sources_.push(source);
    return source;
  }

  // Helper methods
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
        headFromEye: getGFXTransformFromPosition(-0.032, 0, 0),
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
        headFromEye: getGFXTransformFromPosition(0.032, 0, 0),
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
      headFromEye: getGFXTransformFromPosition(offset[0], offset[1], offset[2]),
      renderWidth: fakeXRViewInit.resolution.width,
      renderHeight: fakeXRViewInit.resolution.height
    };
  }

  setFeatures(supportedFeatures) {
    function convertFeatureToMojom(feature) {
      switch (feature) {
        case "viewer":
          return device.mojom.XRSessionFeature.REF_SPACE_VIEWER;
        case "local":
          return device.mojom.XRSessionFeature.REF_SPACE_LOCAL;
        case "local-floor":
          return device.mojom.XRSessionFeature.REF_SPACE_LOCAL_FLOOR;
        case "bounded-floor":
          return device.mojom.XRSessionFeature.REF_SPACE_BOUNDED_FLOOR;
        case "unbounded":
          return device.mojom.XRSessionFeature.REF_SPACE_UNBOUNDED;
        default:
          return device.mojom.XRSessionFeature.INVALID;
      }
    }

    this.supportedFeatures_ = [];

    for (let i = 0; i < supportedFeatures.length; i++) {
      let feature = convertFeatureToMojom(supportedFeatures[i]);
      if (feature !== device.mojom.XRSessionFeature.INVALID) {
        this.supportedFeatures_.push(feature);
      }
    }
  }

  // These methods are intended to be used by MockXRInputSource only.
  addInputSource(source) {
    let index = this.input_sources_.indexOf(source);
    if (index == -1) {
      this.input_sources_.push(source);
    }
  }

  removeInputSource(source) {
    let index = this.input_sources_.indexOf(source);
    if (index >= 0) {
      this.input_sources_.splice(index, 1);
    }
  }

  // Mojo function implementations.

  // XRFrameDataProvider implementation.
  getFrameData() {
    if (this.pose_) {
      this.pose_.poseIndex++;
      this.pose_.poseReset = this.send_pose_reset_;
      this.send_pose_reset_ = false;

      // Setting the input_state to null tests a slightly different path than
      // the browser tests where if the last input source is removed, the device
      // code always sends up an empty array, but it's also valid mojom to send
      // up a null array.
      if (this.input_sources_.length > 0) {
        this.pose_.inputState = [];
        for (let i = 0; i < this.input_sources_.length; i++) {
          this.pose_.inputState.push(this.input_sources_[i].getInputSourceState());
        }
      } else {
        this.pose_.inputState = null;
      }
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

        let enabled_features = [];
        for(let i = 0; i < sessionOptions.requiredFeatures.length; i++) {
          if (this.supportedFeatures_.indexOf(sessionOptions.requiredFeatures[i]) !== -1) {
            enabled_features.push(sessionOptions.requiredFeatures[i]);
          } else {
            return Promise.resolve({session: null});
          }
        }

        for (let i =0; i < sessionOptions.optionalFeatures.length; i++) {
          if (this.supportedFeatures_.indexOf(sessionOptions.optionalFeatures[i]) !== -1) {
            enabled_features.push(sessionOptions.optionalFeatures[i]);
          }
        }

        return Promise.resolve({
          session: {
            submitFrameSink: submit_frame_sink,
            dataProvider: dataProviderPtr,
            clientRequest: clientRequest,
            displayInfo: this.displayInfo_,
            enabledFeatures: enabled_features,
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

class MockXRInputSource {
  constructor(fakeInputSourceInit, id, pairedDevice) {
    this.source_id_ = id;
    this.pairedDevice_ = pairedDevice;
    this.handedness_ = fakeInputSourceInit.handedness;
    this.target_ray_mode_ = fakeInputSourceInit.targetRayMode;
    this.setPointerOrigin(fakeInputSourceInit.pointerOrigin);
    this.setProfiles(fakeInputSourceInit.profiles);

    this.primary_input_pressed_ = false;
    if (fakeInputSourceInit.selectionStarted != null) {
      this.primary_input_pressed_ = fakeInputSourceInit.selectionStarted;
    }

    this.primary_input_clicked_ = false;
    if (fakeInputSourceInit.selectionClicked != null) {
      this.primary_input_clicked_ = fakeInputSourceInit.selectionClicked;
    }

    this.grip_ = null;
    if (fakeInputSourceInit.gripOrigin != null) {
      this.setGripOrigin(fakeInputSourceInit.gripOrigin);
    }

    // This properly handles if supportedButtons were not specified.
    this.setSupportedButtons(fakeInputSourceInit.supportedButtons);

    this.emulated_position_ = false;
    this.desc_dirty_ = true;
  }

  // Webxr-test-api
  setHandedness(handedness) {
    if (this.handedness_ != handedness) {
      this.desc_dirty_ = true;
      this.handedness_ = handedness;
    }
  }

  setTargetRayMode(targetRayMode) {
    if (this.target_ray_mode_ != targetRayMode) {
      this.desc_dirty_ = true;
      this.target_ray_mode_ = targetRayMode;
    }
  }

  setProfiles(profiles) {
    this.desc_dirty_ = true;
    this.profiles_ = profiles;
  }

  setGripOrigin(transform, emulatedPosition = false) {
    this.grip_ = new gfx.mojom.Transform();
    this.grip_.matrix = getMatrixFromTransform(transform);
    this.emulated_position_ = emulatedPosition;
  }

  clearGripOrigin() {
    if (this.grip_ != null) {
      this.grip_ = null;
      this.emulated_position_ = false;
    }
  }

  setPointerOrigin(transform, emulatedPosition = false) {
    this.desc_dirty_ = true;
    this.pointer_offset_ = new gfx.mojom.Transform();
    this.pointer_offset_.matrix = getMatrixFromTransform(transform);
  }

  disconnect() {
    this.pairedDevice_.removeInputSource(this);
  }

  reconnect() {
    this.pairedDevice_.addInputSource(this);
  }

  startSelection() {
    this.primary_input_pressed_ = true;
    if (this.gamepad_) {
      this.gamepad_.buttons[0].pressed = true;
      this.gamepad_.buttons[0].touched = true;
    }
  }

  endSelection() {
    if (!this.primary_input_pressed_) {
      throw new Error("Attempted to end selection which was not started");
    }

    this.primary_input_pressed_ = false;
    this.primary_input_clicked_ = true;

    if (this.gamepad_) {
      this.gamepad_.buttons[0].pressed = false;
      this.gamepad_.buttons[0].touched = false;
    }
  }

  simulateSelect() {
    this.primary_input_clicked_ = true;
  }

  setSupportedButtons(supportedButtons) {
    this.gamepad_ = null;
    this.supported_buttons_ = [];

    // If there are no supported buttons, we can stop now.
    if (supportedButtons == null || supportedButtons.length < 1) {
      return;
    }

    let supported_button_map = {};
    this.gamepad_ = this.getEmptyGamepad();
    for (let i = 0; i < supportedButtons.length; i++) {
      let buttonType = supportedButtons[i].buttonType;
      this.supported_buttons_.push(buttonType);
      supported_button_map[buttonType] = supportedButtons[i];
    }

    // Let's start by building the button state in order of priority:
    // Primary button is index 0.
    this.gamepad_.buttons.push({
      pressed: this.primary_input_pressed_,
      touched: this.primary_input_pressed_,
      value: this.primary_input_pressed_ ? 1.0 : 0.0
    });

    // Now add the rest of our buttons
    this.addGamepadButton(supported_button_map['grip']);
    this.addGamepadButton(supported_button_map['touchpad']);
    this.addGamepadButton(supported_button_map['thumbstick']);
    this.addGamepadButton(supported_button_map['optional-button']);
    this.addGamepadButton(supported_button_map['optional-thumbstick']);

    // Finally, back-fill placeholder buttons/axes
    for (let i = 0; i < this.gamepad_.buttons.length; i++) {
      if (this.gamepad_.buttons[i] == null) {
        this.gamepad_.buttons[i] = {
          pressed: false,
          touched: false,
          value: 0
        }
      }
    }

    for (let i=0; i < this.gamepad_.axes.length; i++) {
      if (this.gamepad_.axes[i] == null) {
        this.gamepad_.axes[i] = 0;
      }
    }
  }

  updateButtonState(buttonState) {
    if (this.supported_buttons_.indexOf(buttonState.buttonType) == -1) {
      throw new Error("Tried to update state on an unsupported button");
    }

    let buttonIndex = this.getButtonIndex(buttonState.buttonType);
    let axesStartIndex = this.getAxesStartIndex(buttonState.buttonType);

    if (buttonIndex == -1) {
      throw new Error("Unknown Button Type!");
    }

    this.gamepad_.buttons[buttonIndex].pressed = buttonState.pressed;
    this.gamepad_.buttons[buttonIndex].touched = buttonState.touched;
    this.gamepad_.buttons[buttonIndex].value = buttonState.pressedValue;

    if (axesStartIndex != -1) {
      this.gamepad_.axes[axesStartIndex] = buttonState.xValue == null ? 0.0 : buttonState.xValue;
      this.gamepad_.axes[axesStartIndex + 1] = buttonState.yValue == null ? 0.0 : buttonState.yValue;
    }
  }

  // Helpers for Mojom
  getInputSourceState() {
    let input_state = new device.mojom.XRInputSourceState();

    input_state.sourceId = this.source_id_;

    input_state.primaryInputPressed = this.primary_input_pressed_;
    input_state.primaryInputClicked = this.primary_input_clicked_;

    input_state.grip = this.grip_;

    input_state.gamepad = this.gamepad_;

    if (this.desc_dirty_) {
      let input_desc = new device.mojom.XRInputSourceDescription();

      input_desc.emulatedPosition = this.emulated_position_;

      switch (this.target_ray_mode_) {
        case 'gaze':
          input_desc.targetRayMode = device.mojom.XRTargetRayMode.GAZING;
          break;
        case 'tracked-pointer':
          input_desc.targetRayMode = device.mojom.XRTargetRayMode.POINTING;
          break;
      }

      switch (this.handedness_) {
        case 'left':
          input_desc.handedness = device.mojom.XRHandedness.LEFT;
          break;
        case 'right':
          input_desc.handedness = device.mojom.XRHandedness.RIGHT;
          break;
        default:
          input_desc.handedness = device.mojom.XRHandedness.NONE;
          break;
      }

      input_desc.pointerOffset = this.pointer_offset_;

      input_desc.profiles = this.profiles_;

      input_state.description = input_desc;

      this.desc_dirty_ = false;
    }

    return input_state;
  }

  getEmptyGamepad() {
    // Mojo complains if some of the properties on Gamepad are null, so set
    // everything to reasonable defaults that tests can override.
    let gamepad = new device.mojom.Gamepad();
    gamepad.connected = true;
    gamepad.id = "";
    gamepad.timestamp = 0;
    gamepad.axes = [];
    gamepad.buttons = [];
    gamepad.mapping = "xr-standard";
    gamepad.display_id = 0;

    switch (this.handedness_) {
      case 'left':
      gamepad.hand = device.mojom.GamepadHand.GamepadHandLeft;
      break;
      case 'right':
      gamepad.hand = device.mojom.GamepadHand.GamepadHandRight;
      break;
      default:
      gamepad.hand = device.mojom.GamepadHand.GamepadHandNone;
      break;
    }

    return gamepad;
  }

  addGamepadButton(buttonState) {
    if (buttonState == null) {
      return;
    }

    let buttonIndex = this.getButtonIndex(buttonState.buttonType);
    let axesStartIndex = this.getAxesStartIndex(buttonState.buttonType);

    if (buttonIndex == -1) {
      throw new Error("Unknown Button Type!");
    }

    this.gamepad_.buttons[buttonIndex] = {
      pressed: buttonState.pressed,
      touched: buttonState.touched,
      value: buttonState.pressedValue
    };

    // Add x/y value if supported.
    if (axesStartIndex != -1) {
      this.gamepad_.axes[axesStartIndex] = (buttonState.xValue == null ? 0.0 : buttonSate.xValue);
      this.gamepad_.axes[axesStartIndex + 1] = (buttonState.yValue == null ? 0.0 : buttonSate.yValue);
    }
  }

  // General Helper methods
  getButtonIndex(buttonType) {
    switch (buttonType) {
      case 'grip':
        return 1;
      case 'touchpad':
        return 2;
      case 'thumbstick':
        return 3;
      case 'optional-button':
        return 4;
      case 'optional-thumbstick':
        return 5;
      default:
        return -1;
    }
  }

  getAxesStartIndex(buttonType) {
    switch (buttonType) {
      case 'touchpad':
        return 0;
      case 'thumbstick':
        return 2;
      case 'optional-thumbstick':
        return 4;
      default:
        return -1;
    }
  }
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
