/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-body-element
[HTMLConstructor]
interface HTMLBodyElement : HTMLElement {
  // also has obsolete members
};
HTMLBodyElement implements WindowEventHandlers;

// https://html.spec.whatwg.org/multipage/#HTMLBodyElement-partial
partial interface HTMLBodyElement {
    [TreatNullAs=EmptyString] attribute DOMString text;

  // https://github.com/servo/servo/issues/8715
  //[TreatNullAs=EmptyString] attribute DOMString link;

  // https://github.com/servo/servo/issues/8716
  //[TreatNullAs=EmptyString] attribute DOMString vLink;

  // https://github.com/servo/servo/issues/8717
  //[TreatNullAs=EmptyString] attribute DOMString aLink;

    [TreatNullAs=EmptyString] attribute DOMString bgColor;
  attribute DOMString background;
};
