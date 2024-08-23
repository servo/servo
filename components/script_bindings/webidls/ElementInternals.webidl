/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#elementinternals
[Exposed=Window]
interface ElementInternals {
  // Form-associated custom elements

  [Throws] undefined setFormValue((File or USVString or FormData)? value,
                    optional (File or USVString or FormData)? state);

  [Throws] readonly attribute HTMLFormElement? form;

  // flags shouldn't be optional here, #25704
  [Throws] undefined setValidity(optional ValidityStateFlags flags = {},
                   optional DOMString message,
                   optional HTMLElement anchor);
  [Throws] readonly attribute boolean willValidate;
  [Throws] readonly attribute ValidityState validity;
  [Throws] readonly attribute DOMString validationMessage;
  [Throws] boolean checkValidity();
  [Throws] boolean reportValidity();

  [Throws] readonly attribute NodeList labels;
};

// https://html.spec.whatwg.org/multipage/#elementinternals
dictionary ValidityStateFlags {
  boolean valueMissing = false;
  boolean typeMismatch = false;
  boolean patternMismatch = false;
  boolean tooLong = false;
  boolean tooShort = false;
  boolean rangeUnderflow = false;
  boolean rangeOverflow = false;
  boolean stepMismatch = false;
  boolean badInput = false;
  boolean customError = false;
};

