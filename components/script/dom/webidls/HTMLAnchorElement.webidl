/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.whatwg.org/specs/web-apps/current-work/#the-a-element
 * https://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// https://www.whatwg.org/html/#htmlanchorelement
interface HTMLAnchorElement : HTMLElement {
  //         attribute DOMString target;
  //         attribute DOMString download;
  //[PutForwards=value] attribute DOMSettableTokenList ping;
  //         attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  //         attribute DOMString hreflang;
  //         attribute DOMString type;

  [Pure]
           attribute DOMString text;

  // also has obsolete members
};
//HTMLAnchorElement implements URLUtils;

// https://www.whatwg.org/html/#HTMLAnchorElement-partial
partial interface HTMLAnchorElement {
  attribute DOMString coords;
  //         attribute DOMString charset;
  attribute DOMString name;
  attribute DOMString rev;
  attribute DOMString shape;
};
