/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablesectionelement
[HTMLConstructor]
interface HTMLTableSectionElement : HTMLElement {
  readonly attribute HTMLCollection rows;
  [Throws]
  HTMLElement insertRow(optional long index = -1);
  [Throws]
  void deleteRow(long index);

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableSectionElement-partial
partial interface HTMLTableSectionElement {
  //         attribute DOMString align;
  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute DOMString vAlign;
};
