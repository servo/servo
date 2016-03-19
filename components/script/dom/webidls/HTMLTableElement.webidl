/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltableelement
interface HTMLTableElement : HTMLElement {
           attribute HTMLTableCaptionElement? caption;
  HTMLElement createCaption();
  void deleteCaption();
  //         attribute HTMLTableSectionElement? tHead;
  //HTMLElement createTHead();
  //void deleteTHead();
  //         attribute HTMLTableSectionElement? tFoot;
  //HTMLElement createTFoot();
  //void deleteTFoot();
  //readonly attribute HTMLCollection tBodies;
  HTMLTableSectionElement createTBody();
  readonly attribute HTMLCollection rows;
  //HTMLElement insertRow(optional long index = -1);
  //void deleteRow(long index);
  //         attribute boolean sortable;
  //void stopSorting();

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableElement-partial
partial interface HTMLTableElement {
  //         attribute DOMString align;
  //         attribute DOMString border;
  //         attribute DOMString frame;
  //         attribute DOMString rules;
  //         attribute DOMString summary;
  attribute DOMString width;

  [TreatNullAs=EmptyString] attribute DOMString bgColor;
  //[TreatNullAs=EmptyString] attribute DOMString cellPadding;
  //[TreatNullAs=EmptyString] attribute DOMString cellSpacing;
};
