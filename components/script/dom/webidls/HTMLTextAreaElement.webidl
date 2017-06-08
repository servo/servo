/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltextareaelement
[HTMLConstructor]
interface HTMLTextAreaElement : HTMLElement {
  //         attribute DOMString autocomplete;
  //         attribute boolean autofocus;
             [SetterThrows]
             attribute unsigned long cols;
  //         attribute DOMString dirName;
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  //         attribute DOMString inputMode;
  //         attribute long maxLength;
  //         attribute long minLength;
           attribute DOMString name;
           attribute DOMString placeholder;
           attribute boolean readOnly;
           attribute boolean required;
           [SetterThrows]
           attribute unsigned long rows;
           attribute DOMString wrap;

  readonly attribute DOMString type;
           attribute DOMString defaultValue;
  [TreatNullAs=EmptyString] attribute DOMString value;
  //readonly attribute unsigned long textLength;

  //readonly attribute boolean willValidate;
  //readonly attribute ValidityState validity;
  //readonly attribute DOMString validationMessage;
  //boolean checkValidity();
  //boolean reportValidity();
  //void setCustomValidity(DOMString error);

  readonly attribute NodeList labels;

  //void select();
           attribute unsigned long selectionStart;
           attribute unsigned long selectionEnd;
           attribute DOMString selectionDirection;
  //void setRangeText(DOMString replacement);
  //void setRangeText(DOMString replacement, unsigned long start, unsigned long end,
  //                  optional SelectionMode selectionMode = "preserve");
  void setSelectionRange(unsigned long start, unsigned long end, optional DOMString direction);
};
