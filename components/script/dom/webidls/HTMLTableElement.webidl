/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltableelement
[HTMLConstructor]
interface HTMLTableElement : HTMLElement {
  [CEReactions]
           attribute HTMLTableCaptionElement? caption;
  HTMLTableCaptionElement createCaption();
  [CEReactions]
  void deleteCaption();

  [CEReactions, SetterThrows]
           attribute HTMLTableSectionElement? tHead;
  HTMLTableSectionElement createTHead();
  [CEReactions]
  void deleteTHead();

  [CEReactions, SetterThrows]
           attribute HTMLTableSectionElement? tFoot;
  HTMLTableSectionElement createTFoot();
  [CEReactions]
  void deleteTFoot();

  readonly attribute HTMLCollection tBodies;
  HTMLTableSectionElement createTBody();

  readonly attribute HTMLCollection rows;
  [Throws] HTMLTableRowElement insertRow(optional long index = -1);
  [CEReactions, Throws] void deleteRow(long index);

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableElement-partial
partial interface HTMLTableElement {
  // [CEReactions]
  //          attribute DOMString align;
  // [CEReactions]
  //          attribute DOMString border;
  // [CEReactions]
  //          attribute DOMString frame;
  // [CEReactions]
  //          attribute DOMString rules;
  // [CEReactions]
  //          attribute DOMString summary;
  [CEReactions]
  attribute DOMString width;

  [CEReactions, TreatNullAs=EmptyString]
           attribute DOMString bgColor;
  // [CEReactions, TreatNullAs=EmptyString]
  //          attribute DOMString cellPadding;
  // [CEReactions, TreatNullAs=EmptyString]
  //          attribute DOMString cellSpacing;
};
