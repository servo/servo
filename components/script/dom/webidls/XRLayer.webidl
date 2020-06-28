/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/layers/#xrlayertype
[SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
interface XRLayer {
//  attribute boolean blendTextureSourceAlpha;
//  attribute boolean chromaticAberrationCorrection;

  void destroy();
};
//
// TODO: Implement the layer types
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.enabled"]
// interface XRProjectionLayer : XRLayer {
//   readonly attribute boolean ignoreDepthValues;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRQuadLayer : XRLayer {
//   readonly attribute XRLayerLayout layout;
//   attribute XRRigidTransform transform;
//
//   attribute float width;
//   attribute float height;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRCylinderLayer : XRLayer {
//   readonly attribute XRLayerLayout layout;
//   attribute XRReferenceSpace referenceSpace;
//
//   attribute XRRigidTransform transform;
//   attribute float radius;
//   attribute float centralAngle;
//   attribute float aspectRatio;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XREquirectLayer : XRLayer {
//   readonly attribute XRLayerLayout layout;
//   attribute XRReferenceSpace referenceSpace;
//
//   attribute XRRigidTransform transform;
//   attribute float radius;
//   attribute float scaleX;
//   attribute float scaleY;
//   attribute float biasX;
//   attribute float biasY;
// };
//
// [SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
// interface XRCubeLayer : XRLayer {
//   readonly attribute XRLayerLayout layout;
//   attribute XRReferenceSpace referenceSpace;
//
//   attribute DOMPoint orientation;
// };
