/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmlbuttonelement
interface HTMLButtonElement : HTMLElement {
  //         attribute boolean autofocus;
           attribute boolean disabled;
  //readonly attribute HTMLFormElement? form;
  //         attribute DOMString formAction;
  //         attribute DOMString formEnctype;
  //         attribute DOMString formMethod;
  //         attribute boolean formNoValidate;
  //         attribute DOMString formTarget;
  //         attribute DOMString name;
           attribute DOMString type;
  //         attribute DOMString value;
  //         attribute HTMLMenuElement? menu;

  //readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  //readonly attribute DOMString validationMessage;
  //boolean checkValidity();
  //boolean reportValidity();
  //void setCustomValidity(DOMString error);

  //readonly attribute NodeList labels;
};
