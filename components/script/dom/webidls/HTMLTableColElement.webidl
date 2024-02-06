/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablecolelement
[Exposed=Window]
interface HTMLTableColElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
            attribute unsigned long span;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableColElement-partial
partial interface HTMLTableColElement {
  // [CEReactions]
  //          attribute DOMString align;
  // [CEReactions]
  //          attribute DOMString ch;
  // [CEReactions]
  //          attribute DOMString chOff;
  // [CEReactions]
  //          attribute DOMString vAlign;
  // [CEReactions]
  //          attribute DOMString width;
};
