/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlareaelement
[HTMLConstructor]
interface HTMLAreaElement : HTMLElement {
  //         attribute DOMString alt;
  //         attribute DOMString coords;
  //         attribute DOMString shape;
  //         attribute DOMString target;
  //         attribute DOMString download;
  //         attribute USVString ping;
  //         attribute DOMString rel;
  readonly attribute DOMTokenList relList;
  // hreflang and type are not reflected
};
//HTMLAreaElement implements HTMLHyperlinkElementUtils;

// https://html.spec.whatwg.org/multipage/#HTMLAreaElement-partial
partial interface HTMLAreaElement {
  //         attribute boolean noHref;
};
