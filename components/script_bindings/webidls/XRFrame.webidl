/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/webxr/#xrframe-interface

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRFrame {
  [SameObject] readonly attribute XRSession session;
  readonly attribute DOMHighResTimeStamp predictedDisplayTime;

  [Throws] XRViewerPose? getViewerPose(XRReferenceSpace referenceSpace);
  [Throws] XRPose? getPose(XRSpace space, XRSpace baseSpace);

  // WebXR Hand Input
  [Pref="dom_webxr_hands_enabled", Throws]
  XRJointPose? getJointPose(XRJointSpace joint, XRSpace baseSpace);
  [Pref="dom_webxr_hands_enabled", Throws]
  boolean fillJointRadii(sequence<XRJointSpace> jointSpaces, Float32Array radii);

  [Pref="dom_webxr_hands_enabled", Throws]
  boolean fillPoses(sequence<XRSpace> spaces, XRSpace baseSpace, Float32Array transforms);

  // WebXR Hit Test
  sequence<XRHitTestResult> getHitTestResults(XRHitTestSource hitTestSource);
};
