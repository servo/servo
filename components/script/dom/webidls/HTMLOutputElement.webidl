/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmloutputelement
interface HTMLOutputElement : HTMLElement {
  //[PutForwards=value] readonly attribute DOMSettableTokenList htmlFor;
  readonly attribute HTMLFormElement? form;
  //         attribute DOMString name;

  //readonly attribute DOMString type;
  //         attribute DOMString defaultValue;
  //         attribute DOMString value;

  //readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  //readonly attribute DOMString validationMessage;
  //boolean checkValidity();
  //boolean reportValidity();
  //void setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
