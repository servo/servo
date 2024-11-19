/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlheadingelement
[Exposed=Window]
interface HTMLHeadingElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLHeadingElement-partial
partial interface HTMLHeadingElement {
  // [CEReactions]
  //         attribute DOMString align;
};
