/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmltablerowelement
interface HTMLTableRowElement : HTMLElement {
  //readonly attribute long rowIndex;
  //readonly attribute long sectionRowIndex;
  //readonly attribute HTMLCollection cells;
  //HTMLElement insertCell(optional long index = -1);
  //void deleteCell(long index);

  // also has obsolete members
};

// http://www.whatwg.org/html/#HTMLTableRowElement-partial
partial interface HTMLTableRowElement {
  //         attribute DOMString align;
  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute DOMString vAlign;

  //[TreatNullAs=EmptyString] attribute DOMString bgColor;
};
