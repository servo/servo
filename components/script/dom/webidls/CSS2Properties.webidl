/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/
 *
 */

interface CSS2Properties : CSSStyleDeclaration {
  [TreatNullAs=EmptyString] attribute DOMString color;
  [TreatNullAs=EmptyString] attribute DOMString display;
  [TreatNullAs=EmptyString] attribute DOMString background;
  [TreatNullAs=EmptyString] attribute DOMString backgroundColor;
  [TreatNullAs=EmptyString] attribute DOMString backgroundPosition;
  [TreatNullAs=EmptyString] attribute DOMString backgroundRepeat;
  [TreatNullAs=EmptyString] attribute DOMString backgroundImage;
  [TreatNullAs=EmptyString] attribute DOMString backgroundAttachment;
  [TreatNullAs=EmptyString] attribute DOMString width;
  [TreatNullAs=EmptyString] attribute DOMString height;
};
