/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/#the-body-element
interface HTMLBodyElement : HTMLElement {
  // also has obsolete members
};
HTMLBodyElement implements WindowEventHandlers;

// https://html.spec.whatwg.org/multipage/#HTMLBodyElement-partial
partial interface HTMLBodyElement {
    [TreatNullAs=EmptyString] attribute DOMString text;
  //[TreatNullAs=EmptyString] attribute DOMString link;
  //[TreatNullAs=EmptyString] attribute DOMString vLink;
  //[TreatNullAs=EmptyString] attribute DOMString aLink;
    [TreatNullAs=EmptyString] attribute DOMString bgColor;
  //                          attribute DOMString background;
};
