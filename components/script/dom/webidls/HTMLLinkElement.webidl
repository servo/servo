/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmllinkelement
interface HTMLLinkElement : HTMLElement {
           attribute DOMString href;
  //         attribute DOMString crossOrigin;
           attribute DOMString rel;
  readonly attribute DOMTokenList relList;
           attribute DOMString media;
           attribute DOMString hreflang;
           attribute DOMString type;
           attribute DOMString integrity;
  // [SameObject, PutForwards=value] readonly attribute DOMTokenList sizes;

  // also has obsolete members
};
HTMLLinkElement implements LinkStyle;

// https://html.spec.whatwg.org/multipage/#HTMLLinkElement-partial
partial interface HTMLLinkElement {
  attribute DOMString charset;
  attribute DOMString rev;
  attribute DOMString target;
};
