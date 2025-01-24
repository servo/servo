/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_WEBXR

// https://immersive-web.github.io/webxr/#xrwebgllayer-interface

typedef (WebGLRenderingContext or
         WebGL2RenderingContext) XRWebGLRenderingContext;

dictionary XRWebGLLayerInit {
  boolean antialias = true;
  boolean depth = true;
  boolean stencil = false;
  boolean alpha = true;
  boolean ignoreDepthValues = false;
  double framebufferScaleFactor = 1.0;
};

[SecureContext, Exposed=Window, Pref="dom_webxr_enabled"]
interface XRWebGLLayer: XRLayer {
  [Throws] constructor(XRSession session,
              XRWebGLRenderingContext context,
              optional XRWebGLLayerInit layerInit = {});
  // Attributes
  readonly attribute boolean antialias;
  readonly attribute boolean ignoreDepthValues;
  attribute float? fixedFoveation;

  [SameObject] readonly attribute WebGLFramebuffer? framebuffer;
  readonly attribute unsigned long framebufferWidth;
  readonly attribute unsigned long framebufferHeight;

  // Methods
  XRViewport? getViewport(XRView view);

  // Static Methods
  static double getNativeFramebufferScaleFactor(XRSession session);
};

partial interface mixin WebGLRenderingContextBase {
    [Pref="dom_webxr_enabled"] Promise<undefined> makeXRCompatible();
};
