/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlfieldsetelement
[Exposed=Window]
interface HTMLFieldSetElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  [CEReactions]
           attribute DOMString name;

  readonly attribute DOMString type;

  [SameObject] readonly attribute HTMLCollection elements;

  readonly attribute boolean willValidate;
  [SameObject] readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);
};
