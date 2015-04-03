/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//  https://www.whatwg.org/html/#htmlcanvaselement
typedef (CanvasRenderingContext2D or WebGLRenderingContext) RenderingContext;

interface HTMLCanvasElement : HTMLElement {
  [Pure]
           attribute unsigned long width;
  [Pure]
           attribute unsigned long height;

  RenderingContext? getContext(DOMString contextId);
  //boolean probablySupportsContext(DOMString contextId, any... arguments);

  //void setContext(RenderingContext context);
  //CanvasProxy transferControlToProxy();

  //DOMString toDataURL(optional DOMString type, any... arguments);
  //void toBlob(FileCallback? _callback, optional DOMString type, any... arguments);
};
