/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-offscreencanvas-interface
typedef (OffscreenCanvasRenderingContext2D
  or ImageBitmapRenderingContext
  or WebGLRenderingContext
  or WebGL2RenderingContext) OffscreenRenderingContext;

dictionary ImageEncodeOptions {
  DOMString type = "image/png";
  unrestricted double quality;
};

//enum OffscreenRenderingContextId { "2d", "bitmaprenderer", "webgl", "webgl2" };

[Exposed=(Window,Worker), Transferable, Pref="dom_offscreen_canvas_enabled"]
interface OffscreenCanvas : EventTarget {
  [Throws] constructor([EnforceRange] unsigned long long width, [EnforceRange] unsigned long long height);
  attribute [EnforceRange] unsigned long long width;
  attribute [EnforceRange] unsigned long long height;

  [Throws] OffscreenRenderingContext? getContext(DOMString contextId, optional any options = null);
  [Throws] ImageBitmap transferToImageBitmap();
  Promise<Blob> convertToBlob(optional ImageEncodeOptions options = {});
};
