/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablerowelement
[HTMLConstructor]
interface HTMLTableRowElement : HTMLElement {
  readonly attribute long rowIndex;
  readonly attribute long sectionRowIndex;
  readonly attribute HTMLCollection cells;
  [Throws]
  HTMLElement insertCell(optional long index = -1);
  [Throws]
  void deleteCell(long index);

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableRowElement-partial
partial interface HTMLTableRowElement {
  //         attribute DOMString align;
  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute DOMString vAlign;

  [TreatNullAs=EmptyString] attribute DOMString bgColor;
};
