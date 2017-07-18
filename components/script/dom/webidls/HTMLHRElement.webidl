/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlhrelement
[HTMLConstructor]
interface HTMLHRElement : HTMLElement {
  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLHRElement-partial
partial interface HTMLHRElement {
  [CEReactions]
  attribute DOMString align;
  [CEReactions]
  attribute DOMString color;
  // [CEReactions]
  // attribute boolean noShade;
  // [CEReactions]
  // attribute DOMString size;
  [CEReactions]
  attribute DOMString width;
};
