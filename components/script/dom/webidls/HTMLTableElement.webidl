/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltableelement
[HTMLConstructor]
interface HTMLTableElement : HTMLElement {
           attribute HTMLTableCaptionElement? caption;
  HTMLTableCaptionElement createCaption();
  void deleteCaption();
  [SetterThrows]
           attribute HTMLTableSectionElement? tHead;
  HTMLTableSectionElement createTHead();
  void deleteTHead();
  [SetterThrows]
           attribute HTMLTableSectionElement? tFoot;
  HTMLTableSectionElement createTFoot();
  void deleteTFoot();
  readonly attribute HTMLCollection tBodies;
  HTMLTableSectionElement createTBody();
  readonly attribute HTMLCollection rows;
  [Throws] HTMLTableRowElement insertRow(optional long index = -1);
  [Throws] void deleteRow(long index);

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
