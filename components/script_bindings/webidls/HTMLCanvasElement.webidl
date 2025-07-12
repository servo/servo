/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlcanvaselement
typedef (CanvasRenderingContext2D
  or ImageBitmapRenderingContext
  or WebGLRenderingContext
  or WebGL2RenderingContext
  or GPUCanvasContext) RenderingContext;

[Exposed=Window]
interface HTMLCanvasElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Pure, SetterThrows] attribute unsigned long width;
  [CEReactions, Pure, SetterThrows] attribute unsigned long height;

  [Throws]
  RenderingContext? getContext(DOMString contextId, optional any options = null);

  [Throws]
  USVString toDataURL(optional DOMString type = "image/png", optional any quality);

  [Throws]
  undefined toBlob(BlobCallback callback, optional DOMString type = "image/png", optional any quality);

  [Throws]
  OffscreenCanvas transferControlToOffscreen();
};

partial interface HTMLCanvasElement {
    [Pref="dom_canvas_capture_enabled"]
    MediaStream captureStream (optional double frameRequestRate);
};

callback BlobCallback = undefined(Blob? blob);
