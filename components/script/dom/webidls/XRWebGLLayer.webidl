/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://immersive-web.github.io/webxr/#xrwebgllayer-interface

// typedef (WebGLRenderingContext or
//          WebGL2RenderingContext) XRWebGLRenderingContext;

typedef WebGLRenderingContext XRWebGLRenderingContext;

dictionary XRWebGLLayerInit {
  boolean antialias = true;
  boolean depth = true;
  boolean stencil = false;
  boolean alpha = true;
  // double framebufferScaleFactor = 1.0;
};

[SecureContext, Exposed=Window, Constructor(XRSession session,
            XRWebGLRenderingContext context,
            optional XRWebGLLayerInit layerInit)]
interface XRWebGLLayer : XRLayer {
  // // Attributes
  readonly attribute XRWebGLRenderingContext context;

  readonly attribute boolean antialias;
  readonly attribute boolean depth;
  readonly attribute boolean stencil;
  readonly attribute boolean alpha;

  // readonly attribute WebGLFramebuffer framebuffer;
  // readonly attribute unsigned long framebufferWidth;
  // readonly attribute unsigned long framebufferHeight;

  // // Methods
  // XRViewport? getViewport(XRView view);
  // void requestViewportScaling(double viewportScaleFactor);

  // // Static Methods
  // static double getNativeFramebufferScaleFactor(XRSession session);
};