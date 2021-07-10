/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/layers/#XRWebGLBindingtype
[SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
interface XRWebGLBinding {
  constructor(XRSession session, XRWebGLRenderingContext context);

//  readonly attribute double nativeProjectionScaleFactor;

  [Throws] XRProjectionLayer createProjectionLayer(XRTextureType textureType,
                                          optional XRProjectionLayerInit init = {});
  [Throws] XRQuadLayer createQuadLayer(XRTextureType textureType,
                              optional XRQuadLayerInit init);
  [Throws] XRCylinderLayer createCylinderLayer(XRTextureType textureType,
                                      optional XRCylinderLayerInit init);
  [Throws] XREquirectLayer createEquirectLayer(XRTextureType textureType,
                                      optional XREquirectLayerInit init);
  [Throws] XRCubeLayer createCubeLayer(optional XRCubeLayerInit init);

  [Throws] XRWebGLSubImage getSubImage(XRCompositionLayer layer, XRFrame frame, optional XREye eye = "none");
  [Throws] XRWebGLSubImage getViewSubImage(XRProjectionLayer layer, XRView view);
};

dictionary XRProjectionLayerInit {
  boolean depth = true;
  boolean stencil = false;
  boolean alpha = true;
  double scaleFactor = 1.0;
};

dictionary XRQuadLayerInit : XRLayerInit {
  XRRigidTransform? transform;
  float width = 1.0;
  float height = 1.0;
  boolean isStatic = false;
};

dictionary XRCylinderLayerInit : XRLayerInit {
  XRRigidTransform? transform;
  float radius = 2.0;
  float centralAngle = 0.78539;
  float aspectRatio = 2.0;
  boolean isStatic = false;
};

dictionary XREquirectLayerInit : XRLayerInit {
  XRRigidTransform? transform;
  float radius = 0;
  float centralHorizontalAngle = 6.28318;
  float upperVerticalAngle = 1.570795;
  float lowerVerticalAngle = -1.570795;
  boolean isStatic = false;
};

dictionary XRCubeLayerInit : XRLayerInit {
  DOMPointReadOnly? orientation;
  boolean isStatic = false;
};

dictionary XRLayerInit {
  required XRSpace space;
  required unsigned long viewPixelWidth;
  required unsigned long viewPixelHeight;
  XRLayerLayout layout = "mono";
  boolean depth = false;
  boolean stencil = false;
  boolean alpha = true;
};

enum XRTextureType {
  "texture",
  "texture-array"
};

enum XRLayerLayout {
  "default",
  "mono",
  "stereo",
  "stereo-left-right",
  "stereo-top-bottom"
};
