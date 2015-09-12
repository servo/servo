/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.whatwg.org/html/#htmllinkelement
interface HTMLLinkElement : HTMLElement {
           attribute DOMString href;
  //         attribute DOMString crossOrigin;
           attribute DOMString rel;
  readonly attribute DOMTokenList relList;
           attribute DOMString media;
           attribute DOMString hreflang;
           attribute DOMString type;
  //[PutForwards=value] readonly attribute DOMSettableTokenList sizes;

  // also has obsolete members
};
//HTMLLinkElement implements LinkStyle;

// https://www.whatwg.org/html/#HTMLLinkElement-partial
partial interface HTMLLinkElement {
  attribute DOMString charset;
  attribute DOMString rev;
  attribute DOMString target;
};
