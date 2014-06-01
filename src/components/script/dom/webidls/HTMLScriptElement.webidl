/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmlscriptelement
interface HTMLScriptElement : HTMLElement {
  //         attribute DOMString src;
  readonly attribute DOMString src;
  //         attribute DOMString type;
  //         attribute DOMString charset;
  //         attribute boolean async;
  //         attribute boolean defer;
  //         attribute DOMString crossOrigin;
  //         attribute DOMString text;

  // also has obsolete members
};

// http://www.whatwg.org/html/#HTMLScriptElement-partial
partial interface HTMLScriptElement {
  //         attribute DOMString event;
  //         attribute DOMString htmlFor;
};
