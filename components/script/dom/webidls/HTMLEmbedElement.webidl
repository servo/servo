/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlembedelement
[HTMLConstructor]
interface HTMLEmbedElement : HTMLElement {
  // [CEReactions]
  //         attribute DOMString src;
  // [CEReactions]
  //         attribute DOMString type;
  // [CEReactions]
  //         attribute DOMString width;
  // [CEReactions]
  //         attribute DOMString height;
  //legacycaller any (any... arguments);

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLEmbedElement-partial
partial interface HTMLEmbedElement {
  // [CEReactions]
  //         attribute DOMString align;
  // [CEReactions]
  //         attribute DOMString name;
};
