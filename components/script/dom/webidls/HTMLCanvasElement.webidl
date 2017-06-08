/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlcanvaselement
typedef (CanvasRenderingContext2D or WebGLRenderingContext) RenderingContext;

[HTMLConstructor]
interface HTMLCanvasElement : HTMLElement {
  [Pure]
           attribute unsigned long width;
  [Pure]
           attribute unsigned long height;

  RenderingContext? getContext(DOMString contextId, any... arguments);
  //boolean probablySupportsContext(DOMString contextId, any... arguments);

  //void setContext(RenderingContext context);
  //CanvasProxy transferControlToProxy();

  [Throws]
  DOMString toDataURL(optional DOMString type, any... arguments);
  //void toBlob(FileCallback? _callback, optional DOMString type, any... arguments);
};
