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
  [CEReactions]
  attribute DOMString target;
  // [CEReactions]
  //       attribute DOMString download;
  // [CEReactions]
  //       attribute USVString ping;
  [CEReactions]
           attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  // [CEReactions]
  //       attribute DOMString hreflang;
  // [CEReactions]
  //       attribute DOMString type;

  [CEReactions, Pure]
           attribute DOMString text;

  // also has obsolete members
};
HTMLAnchorElement implements HTMLHyperlinkElementUtils;

// https://html.spec.whatwg.org/multipage/#HTMLAnchorElement-partial
partial interface HTMLAnchorElement {
  [CEReactions]
  attribute DOMString coords;
  // [CEReactions]
  //          attribute DOMString charset;
  [CEReactions]
  attribute DOMString name;
  [CEReactions]
  attribute DOMString rev;
  [CEReactions]
  attribute DOMString shape;
};
