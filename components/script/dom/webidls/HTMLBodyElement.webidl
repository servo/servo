/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-body-element
[Exposed=Window]
interface HTMLBodyElement : HTMLElement {
  [HTMLConstructor] constructor();

  // also has obsolete members
};
HTMLBodyElement includes WindowEventHandlers;

// https://html.spec.whatwg.org/multipage/#HTMLBodyElement-partial
partial interface HTMLBodyElement {
  [CEReactions] attribute [TreatNullAs=EmptyString] DOMString text;

  // https://github.com/servo/servo/issues/8715
  //[CEReactions, TreatNullAs=EmptyString] attribute DOMString link;

  // https://github.com/servo/servo/issues/8716
  //[CEReactions, TreatNullAs=EmptyString] attribute DOMString vLink;

  // https://github.com/servo/servo/issues/8717
  //[CEReactions, TreatNullAs=EmptyString] attribute DOMString aLink;

  [CEReactions] attribute [TreatNullAs=EmptyString] DOMString bgColor;
  [CEReactions] attribute DOMString background;
};
