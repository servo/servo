/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmloutputelement
[Exposed=Window]
interface HTMLOutputElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [SameObject, PutForwards=value] readonly attribute DOMTokenList htmlFor;
  readonly attribute HTMLFormElement? form;
  // [CEReactions]
  //          attribute DOMString name;

  [Pure] readonly attribute DOMString type;
  [CEReactions]
           attribute DOMString defaultValue;
  [CEReactions]
           attribute DOMString value;

  // readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  // readonly attribute DOMString validationMessage;
  // boolean checkValidity();
  // boolean reportValidity();
  // void setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
