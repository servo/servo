/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlparamelement
[HTMLConstructor]
interface HTMLParamElement : HTMLElement {
  // [CEReactions]
  //         attribute DOMString name;
  // [CEReactions]
  //         attribute DOMString value;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLParamElement-partial
partial interface HTMLParamElement {
  // [CEReactions]
  //         attribute DOMString type;
  // [CEReactions]
  //         attribute DOMString valueType;
};
