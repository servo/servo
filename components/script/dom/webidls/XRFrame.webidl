/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrframe-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRFrame {
  readonly attribute XRSession session;

  [Throws] XRViewerPose? getViewerPose(XRReferenceSpace referenceSpace);
  [Throws] XRPose? getPose(XRSpace space, XRSpace relativeTo);
  [Pref="dom.webxr.hands.enabled", Throws] XRJointPose? getJointPose(XRJointSpace space, XRSpace relativeTo);
  sequence<XRHitTestResult> getHitTestResults(XRHitTestSource hitTestSource);
};
