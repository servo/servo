/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/layers/#XRWebGLBindingtype
[SecureContext, Exposed=Window, Pref="dom.webxr.layers.enabled"]
interface XRWebGLBinding {
  constructor(XRSession session, XRWebGLRenderingContext context);

//  readonly attribute double nativeProjectionScaleFactor;

//  XRProjectionLayer createProjectionLayer(GLenum textureTarget, optional XRProjectionLayerInit init = {});
//  XRQuadLayer createQuadLayer(GLenum textureTarget, XRLayerInit init);
//  XRCylinderLayer createCylinderLayer(GLenum textureTarget, XRLayerInit init);
//  XREquirectLayer createEquirectLayer(GLenum textureTarget, XRLayerInit init);
//  XRCubeLayer createCubeLayer(XRLayerInit init);

  XRWebGLSubImage? getSubImage(XRLayer layer, XRFrame frame); // for mono layers
  XRWebGLSubImage? getViewSubImage(XRLayer layer, XRView view); // for stereo layers
};

dictionary XRProjectionLayerInit {
  boolean depth = true;
  boolean stencil = false;
  boolean alpha = true;
  double scaleFactor = 1.0;
};

dictionary XRLayerInit {
  required unsigned long pixelWidth;
  required unsigned long pixelHeight;
  XRLayerLayout layout = "mono";
  boolean depth = false; // This is a change from typical WebGL initialization, but feels appropriate.
  boolean stencil = false;
  boolean alpha = true;
};

enum XRLayerLayout {
  "mono",
  "stereo",
  "stereo-left-right",
  "stereo-top-bottom"
};

