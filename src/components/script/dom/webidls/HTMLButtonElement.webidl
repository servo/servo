/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-button-element
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// FIXME: servo#707
//interface HTMLFormElement;

// http://www.whatwg.org/specs/web-apps/current-work/#the-button-element
interface HTMLButtonElement : HTMLElement {
  [SetterThrows, Pure]
           attribute boolean autofocus;
  [SetterThrows, Pure]
           attribute boolean disabled;
  [Pure]
  readonly attribute HTMLFormElement? form;
  [SetterThrows, Pure]
           attribute DOMString formAction;
  [SetterThrows, Pure]
           attribute DOMString formEnctype;
  [SetterThrows, Pure]
           attribute DOMString formMethod;
  [SetterThrows, Pure]
           attribute boolean formNoValidate;
  [SetterThrows, Pure]
           attribute DOMString formTarget;
  [SetterThrows, Pure]
           attribute DOMString name;
  [SetterThrows, Pure]
           attribute DOMString type;
  [SetterThrows, Pure]
           attribute DOMString value;
// Not yet implemented:
//           attribute HTMLMenuElement? menu;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);

// Not yet implemented:
//  readonly attribute NodeList labels;
};
