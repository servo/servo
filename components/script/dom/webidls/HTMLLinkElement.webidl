/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmllinkelement
[HTMLConstructor]
interface HTMLLinkElement : HTMLElement {
  [CEReactions]
           attribute DOMString href;
  [CEReactions]
           attribute DOMString? crossOrigin;
  [CEReactions]
           attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  [CEReactions]
           attribute DOMString media;
  [CEReactions]
           attribute DOMString hreflang;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString integrity;
  // [SameObject, PutForwards=value] readonly attribute DOMTokenList sizes;

  // also has obsolete members
};
HTMLLinkElement implements LinkStyle;

// https://html.spec.whatwg.org/multipage/#HTMLLinkElement-partial
partial interface HTMLLinkElement {
  [CEReactions]
  attribute DOMString charset;
  [CEReactions]
  attribute DOMString rev;
  [CEReactions]
  attribute DOMString target;
};
