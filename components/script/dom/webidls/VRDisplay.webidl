/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum VREye {
  "left",
  "right"
};


// https://w3c.github.io/webvr/#interface-vrdisplay
[Pref="dom.webvr.enabled"]
interface VRDisplay : EventTarget {
  readonly attribute boolean isConnected;
  readonly attribute boolean isPresenting;

  /**
   * Dictionary of capabilities describing the VRDisplay.
   */
  [SameObject] readonly attribute VRDisplayCapabilities capabilities;

  /**
   * If this VRDisplay supports room-scale experiences, the optional
   * stage attribute contains details on the room-scale parameters.
   * The stageParameters attribute can not change between null
   * and non-null once the VRDisplay is enumerated; however,
   * the values within VRStageParameters may change after
   * any call to VRDisplay.submitFrame as the user may re-configure
   * their environment at any time.
   */
  readonly attribute VRStageParameters? stageParameters;

  /**
   * Return the current VREyeParameters for the given eye.
   */
  VREyeParameters getEyeParameters(VREye whichEye);

  /**
   * An identifier for this distinct VRDisplay. Used as an
   * association point in the Gamepad API.
   */
  readonly attribute unsigned long displayId;

  /**
   * A display name, a user-readable name identifying it.
   */
  readonly attribute DOMString displayName;

  /**
   * Populates the passed VRFrameData with the information required to render
   * the current frame.
   */
  boolean getFrameData(VRFrameData frameData);

  /**
   * Return a VRPose containing the future predicted pose of the VRDisplay
   * when the current frame will be presented. The value returned will not
   * change until JavaScript has returned control to the browser.
   *
   * The VRPose will contain the position, orientation, velocity,
   * and acceleration of each of these properties.
   */
  [NewObject] VRPose getPose();

  /**
   * Reset the pose for this display, treating its current position and
   * orientation as the "origin/zero" values. VRPose.position,
   * VRPose.orientation, and VRStageParameters.sittingToStandingTransform may be
   * updated when calling resetPose(). This should be called in only
   * sitting-space experiences.
   */
  void resetPose();

  /**
   * z-depth defining the near plane of the eye view frustum
   * enables mapping of values in the render target depth
   * attachment to scene coordinates. Initially set to 0.01.
   */
  attribute double depthNear;

  /**
   * z-depth defining the far plane of the eye view frustum
   * enables mapping of values in the render target depth
   * attachment to scene coordinates. Initially set to 10000.0.
   */
  attribute double depthFar;

  /**
   * The callback passed to `requestAnimationFrame` will be called
   * any time a new frame should be rendered. When the VRDisplay is
   * presenting the callback will be called at the native refresh
   * rate of the HMD. When not presenting this function acts
   * identically to how window.requestAnimationFrame acts. Content should
   * make no assumptions of frame rate or vsync behavior as the HMD runs
   * asynchronously from other displays and at differing refresh rates.
   */
  unsigned long requestAnimationFrame(FrameRequestCallback callback);

  /**
   * Passing the value returned by `requestAnimationFrame` to
   * `cancelAnimationFrame` will unregister the callback.
   */
  void cancelAnimationFrame(unsigned long handle);

  /**
   * Begin presenting to the VRDisplay. Must be called in response to a user gesture.
   * Repeat calls while already presenting will update the VRLayers being displayed.
   * If the number of values in the leftBounds/rightBounds arrays is not 0 or 4 for
   * any of the passed layers the promise is rejected.
   * If the source of any of the layers is not present (null), the promise is rejected.
   */
  Promise<void> requestPresent(sequence<VRLayer> layers);

  /**
   * Stops presenting to the VRDisplay.
   */
  Promise<void> exitPresent();

  /**
   * Get the layers currently being presented.
   */
  //sequence<VRLayer> getLayers();

  /**
   * The VRLayer provided to the VRDisplay will be captured and presented
   * in the HMD. Calling this function has the same effect on the source
   * canvas as any other operation that uses its source image, and canvases
   * created without preserveDrawingBuffer set to true will be cleared.
   */
  void submitFrame();
};
