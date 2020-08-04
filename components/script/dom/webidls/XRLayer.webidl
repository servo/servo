/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrlayer
[SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
interface XRLayer : EventTarget {};

// TODO: Implement the layer types
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRCompositionLayer : XRLayer {
//   readonly attribute XRLayerLayout layout;
//
//   attribute boolean blendTextureSourceAlpha;
//   attribute boolean? chromaticAberrationCorrection;
//   attribute float? fixedFoveation;
//
//   readonly attribute boolean needsRedraw;
//
//   void destroy();
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRProjectionLayer : XRCompositionLayer {
//   readonly attribute boolean ignoreDepthValues;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRQuadLayer : XRCompositionLayer {
//   attribute XRSpace space;
//   attribute XRRigidTransform transform;
//
//   attribute float width;
//   attribute float height;
//
//   // Events
//   attribute EventHandler onredraw;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRCylinderLayer : XRCompositionLayer {
//   attribute XRSpace space;
//   attribute XRRigidTransform transform;
//
//   attribute float radius;
//   attribute float centralAngle;
//   attribute float aspectRatio;
//
//   // Events
//   attribute EventHandler onredraw;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XREquirectLayer : XRCompositionLayer {
//   attribute XRSpace space;
//   attribute XRRigidTransform transform;
//
//   attribute float radius;
//   attribute float centralHorizontalAngle;
//   attribute float upperVerticalAngle;
//   attribute float lowerVerticalAngle;
//
//   // Events
//   attribute EventHandler onredraw;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRCubeLayer : XRCompositionLayer {
//   attribute XRSpace space;
//   attribute DOMPointReadOnly orientation;
//
//   // Events
//   attribute EventHandler onredraw;
// };
