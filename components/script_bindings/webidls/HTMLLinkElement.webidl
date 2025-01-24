/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmllinkelement
[Exposed=Window]
interface HTMLLinkElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute USVString href;
  [CEReactions]
           attribute DOMString? crossOrigin;
  [CEReactions]
           attribute DOMString rel;
  [SameObject, PutForwards=value] readonly attribute DOMTokenList relList;
  [CEReactions]
           attribute DOMString media;
  [CEReactions]
           attribute DOMString hreflang;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString integrity;
  [CEReactions]
           attribute DOMString referrerPolicy;

  // also has obsolete members
};
HTMLLinkElement includes LinkStyle;

// https://html.spec.whatwg.org/multipage/#HTMLLinkElement-partial
partial interface HTMLLinkElement {
  [CEReactions]
  attribute DOMString charset;
  [CEReactions]
  attribute DOMString rev;
  [CEReactions]
  attribute DOMString target;
};
