/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://www.whatwg.org/html/#htmltablecellelement
[Abstract]
interface HTMLTableCellElement : HTMLElement {
             attribute unsigned long colSpan;
  //         attribute unsigned long rowSpan;
  //[PutForwards=value] readonly attribute DOMSettableTokenList headers;
  //readonly attribute long cellIndex;

  // also has obsolete members
};

// https://www.whatwg.org/html/#HTMLTableCellElement-partial
partial interface HTMLTableCellElement {
  //         attribute DOMString align;
  //         attribute DOMString axis;
  //         attribute DOMString height;
  //         attribute DOMString width;

  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute boolean noWrap;
  //         attribute DOMString vAlign;

  //[TreatNullAs=EmptyString] attribute DOMString bgColor;
};
