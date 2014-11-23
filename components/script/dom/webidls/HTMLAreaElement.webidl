/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmlareaelement
interface HTMLAreaElement : HTMLElement {
  //         attribute DOMString alt;
  //         attribute DOMString coords;
  //         attribute DOMString shape;
  //         attribute DOMString target;
  //         attribute DOMString download;
  //[PutForwards=value] attribute DOMSettableTokenList ping;
  //         attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  //         attribute DOMString hreflang;
  //         attribute DOMString type;

  // also has obsolete members
};
//HTMLAreaElement implements URLUtils;

// http://www.whatwg.org/html/#HTMLAreaElement-partial
partial interface HTMLAreaElement {
  //         attribute boolean noHref;
};
