/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#the-a-element
 * https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// https://html.spec.whatwg.org/multipage/#htmlanchorelement
[HTMLConstructor]
interface HTMLAnchorElement : HTMLElement {
  attribute DOMString target;
  //       attribute DOMString download;
  //       attribute USVString ping;
           attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  //       attribute DOMString hreflang;
  //       attribute DOMString type;

  [Pure]
           attribute DOMString text;

  // also has obsolete members
};
HTMLAnchorElement implements HTMLHyperlinkElementUtils;

// https://html.spec.whatwg.org/multipage/#HTMLAnchorElement-partial
partial interface HTMLAnchorElement {
  attribute DOMString coords;
  //         attribute DOMString charset;
  attribute DOMString name;
  attribute DOMString rev;
  attribute DOMString shape;
};
