/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablesectionelement
[Exposed=Window]
interface HTMLTableSectionElement : HTMLElement {
  [HTMLConstructor] constructor();

  readonly attribute HTMLCollection rows;
  [Throws]
  HTMLElement insertRow(optional long index = -1);
  [CEReactions, Throws]
  undefined deleteRow(long index);

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableSectionElement-partial
partial interface HTMLTableSectionElement {
  // [CEReactions]
  //          attribute DOMString align;
  // [CEReactions]
  //          attribute DOMString ch;
  // [CEReactions]
  //          attribute DOMString chOff;
  // [CEReactions]
  //          attribute DOMString vAlign;
};
