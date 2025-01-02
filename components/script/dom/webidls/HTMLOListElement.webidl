/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlolistelement
[Exposed=Window]
interface HTMLOListElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions]
  //         attribute boolean reversed;
  // [CEReactions]
  //         attribute long start;
  // [CEReactions]
  //         attribute DOMString type;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLOListElement-partial
partial interface HTMLOListElement {
  // [CEReactions]
  //         attribute boolean compact;
};
