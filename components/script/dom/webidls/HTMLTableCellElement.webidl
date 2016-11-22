/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablecellelement
[Abstract]
interface HTMLTableCellElement : HTMLElement {
  attribute unsigned long colSpan;
  attribute unsigned long rowSpan;
  //         attribute DOMString headers;
  readonly attribute long cellIndex;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableCellElement-partial
partial interface HTMLTableCellElement {
  //         attribute DOMString align;
  //         attribute DOMString axis;
  //         attribute DOMString height;
  attribute DOMString width;

  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute boolean noWrap;
  //         attribute DOMString vAlign;

  [TreatNullAs=EmptyString] attribute DOMString bgColor;
};
