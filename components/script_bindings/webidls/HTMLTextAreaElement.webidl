/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltextareaelement
[Exposed=Window]
interface HTMLTextAreaElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions]
  //          attribute DOMString autocomplete;
  // [CEReactions]
  //          attribute boolean autofocus;
  [CEReactions, SetterThrows]
           attribute unsigned long cols;
  [CEReactions]
           attribute DOMString dirName;
  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  // [CEReactions]
  //          attribute DOMString inputMode;
  [CEReactions, SetterThrows]
           attribute long maxLength;
  [CEReactions, SetterThrows]
           attribute long minLength;
  [CEReactions]
           attribute DOMString name;
  [CEReactions]
           attribute DOMString placeholder;
  [CEReactions]
           attribute boolean readOnly;
  [CEReactions]
           attribute boolean required;
  [CEReactions, SetterThrows]
           attribute unsigned long rows;
  [CEReactions]
           attribute DOMString wrap;

  readonly attribute DOMString type;
  [CEReactions]
           attribute DOMString defaultValue;
           attribute [LegacyNullToEmptyString] DOMString value;
  readonly attribute unsigned long textLength;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;

  undefined select();
  [SetterThrows]
           attribute unsigned long? selectionStart;
  [SetterThrows]
           attribute unsigned long? selectionEnd;
  [SetterThrows]
           attribute DOMString? selectionDirection;
  [Throws]
           undefined setRangeText(DOMString replacement);
  [Throws]
           undefined setRangeText(DOMString replacement, unsigned long start, unsigned long end,
                             optional SelectionMode selectionMode = "preserve");
  [Throws]
           undefined setSelectionRange(unsigned long start, unsigned long end, optional DOMString direction);
};
