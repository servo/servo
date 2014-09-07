/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmltablesectionelement
interface HTMLTableSectionElement : HTMLElement {
  //readonly attribute HTMLCollection rows;
  //HTMLElement insertRow(optional long index = -1);
  //void deleteRow(long index);

  // also has obsolete members
};

// http://www.whatwg.org/html/#HTMLTableSectionElement-partial
partial interface HTMLTableSectionElement {
  //         attribute DOMString align;
  //         attribute DOMString ch;
  //         attribute DOMString chOff;
  //         attribute DOMString vAlign;
};
