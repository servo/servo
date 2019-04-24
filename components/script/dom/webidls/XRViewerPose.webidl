/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrviewerpose-interface

[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRViewerPose : XRPose {
  // readonly attribute FrozenArray<XRView> views;
  // workaround until we have FrozenArray
  // see https://github.com/servo/servo/issues/10427#issuecomment-449593626
  readonly attribute any views;
};
