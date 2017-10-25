/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

[Pref="dom.offscreen_canvas.enabled"]

typedef (OffscreenCanvasRenderingContext2D or
        WebGLRenderingContext) OffscreenRenderingContext;

dictionary ImageEncodeOptions {
  DOMString type = "image/png";
  unrestricted double quality = 1.0;
};

enum OffscreenRenderingContextType { "2d", "webgl" };

interface OffscreenCanvas : EventTarget {
  attribute unsigned long long width;
  attribute unsigned long long height;

  OffscreenRenderingContext? getContext(OffscreenRenderingContextType contextType, any... arguments);
//  ImageBitmap transferToImageBitmap();
//  Promise<Blob> convertToBlob(optional ImageEncodeOptions options);
};

