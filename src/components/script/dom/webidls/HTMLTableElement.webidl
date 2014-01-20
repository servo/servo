/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLTableElement : HTMLElement {
  /*
           attribute HTMLTableCaptionElement? caption;
  HTMLElement createCaption();
  */
  void deleteCaption();
  /*
           [SetterThrows]
           attribute HTMLTableSectionElement? tHead;
  HTMLElement createTHead();
  */
  void deleteTHead();
  /*
           [SetterThrows]
           attribute HTMLTableSectionElement? tFoot;
  HTMLElement createTFoot();
  */
  void deleteTFoot();
  /*
  readonly attribute HTMLCollection tBodies;
  HTMLElement createTBody();
  readonly attribute HTMLCollection rows;
  [Throws]
  HTMLElement insertRow(optional long index = -1);
  */
  [Throws]
  void deleteRow(long index);
           attribute boolean sortable;
  void stopSorting();
};

partial interface HTMLTableElement {
           [SetterThrows]
           attribute DOMString align;
           [SetterThrows]
           attribute DOMString border;
           [SetterThrows]
           attribute DOMString frame;
           [SetterThrows]
           attribute DOMString rules;
           [SetterThrows]
           attribute DOMString summary;
           [SetterThrows]
           attribute DOMString width;

  [TreatNullAs=EmptyString, SetterThrows] attribute DOMString bgColor;
  [TreatNullAs=EmptyString, SetterThrows] attribute DOMString cellPadding;
  [TreatNullAs=EmptyString, SetterThrows] attribute DOMString cellSpacing;
};
