/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablecellelement
[Exposed=Window]
interface HTMLTableCellElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute unsigned long colSpan;
  [CEReactions]
           attribute unsigned long rowSpan;
  // [CEReactions]
  //          attribute DOMString headers;
  readonly attribute long cellIndex;

  // [CEReactions]
  //          attribute DOMString scope; // only conforming for th elements
  // [CEReactions]
  //          attribute DOMString abbr;  // only conforming for th elements

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableCellElement-partial
partial interface HTMLTableCellElement {
  // [CEReactions]
  //          attribute DOMString align;
  // [CEReactions]
  //          attribute DOMString axis;
  // [CEReactions]
  //          attribute DOMString height;
  [CEReactions]
  attribute DOMString width;

  //          attribute DOMString ch;
  // [CEReactions]
  //          attribute DOMString chOff;
  // [CEReactions]
  //          attribute boolean noWrap;
  // [CEReactions]
  //          attribute DOMString vAlign;

  [CEReactions]
  attribute [LegacyNullToEmptyString] DOMString bgColor;
};
