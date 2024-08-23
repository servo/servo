/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlbuttonelement
[Exposed=Window]
interface HTMLButtonElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions]
  //         attribute boolean autofocus;
  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  [CEReactions]
           attribute DOMString formAction;
  [CEReactions]
           attribute DOMString formEnctype;
  [CEReactions]
           attribute DOMString formMethod;
  [CEReactions]
           attribute boolean formNoValidate;
  [CEReactions]
           attribute DOMString formTarget;
  [CEReactions]
           attribute DOMString name;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString value;
  //         attribute HTMLMenuElement? menu;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
