/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmllegendelement
[HTMLConstructor]
interface HTMLLegendElement : HTMLElement {
  readonly attribute HTMLFormElement? form;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLLegendElement-partial
partial interface HTMLLegendElement {
  //         attribute DOMString align;
};
