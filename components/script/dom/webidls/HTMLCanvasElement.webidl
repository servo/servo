/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlcanvaselement
typedef (CanvasRenderingContext2D
  or WebGLRenderingContext
  or WebGL2RenderingContext
  or [Pref="dom.webgpu.enabled"] GPUCanvasContext) RenderingContext;

[Exposed=Window]
interface HTMLCanvasElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions, Pure] attribute unsigned long width;
  [CEReactions, Pure] attribute unsigned long height;

  RenderingContext? getContext(DOMString contextId, optional any options = null);

  [Throws]
  USVString toDataURL(optional DOMString type, optional any quality);
  
  MediaStream CaptureStream(optional double frameRequestRate);
};