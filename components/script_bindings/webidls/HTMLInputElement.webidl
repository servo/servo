/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlinputelement
[Exposed=Window]
interface HTMLInputElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute DOMString accept;
  [CEReactions]
           attribute DOMString alt;
  // [CEReactions]
  //         attribute DOMString autocomplete;
  // [CEReactions]
  //         attribute boolean autofocus;
  [CEReactions]
           attribute boolean defaultChecked;
           attribute boolean checked;
  [CEReactions]
           attribute DOMString dirName;
  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  attribute FileList? files;
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
  // [CEReactions]
  //          attribute unsigned long height;
           attribute boolean indeterminate;
  // [CEReactions]
  //          attribute DOMString inputMode;
  readonly attribute HTMLDataListElement? list;
  [CEReactions]
           attribute DOMString max;
  [CEReactions, SetterThrows]
           attribute long maxLength;
  [CEReactions]
           attribute DOMString min;
  [CEReactions, SetterThrows]
           attribute long minLength;
  [CEReactions]
           attribute boolean multiple;
  [CEReactions]
           attribute DOMString name;
  [CEReactions]
           attribute DOMString pattern;
  [CEReactions]
           attribute DOMString placeholder;
  [CEReactions]
           attribute boolean readOnly;
  [CEReactions]
           attribute boolean required;
  [CEReactions, SetterThrows]
           attribute unsigned long size;
  [CEReactions]
           attribute USVString src;
  [CEReactions]
           attribute DOMString step;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString defaultValue;
  [CEReactions, SetterThrows]
           attribute [LegacyNullToEmptyString] DOMString value;
  [SetterThrows]
           attribute object? valueAsDate;
  [SetterThrows]
           attribute unrestricted double valueAsNumber;
  // [CEReactions]
  //          attribute unsigned long width;

  [Throws] undefined stepUp(optional long n = 1);
  [Throws] undefined stepDown(optional long n = 1);

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList? labels;

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

  // also has obsolete members

  // Select with file-system paths for testing purpose
  [Pref="dom.testing.htmlinputelement.select_files.enabled"]
  undefined selectFiles(sequence<DOMString> path);
};

// https://html.spec.whatwg.org/multipage/#HTMLInputElement-partial
partial interface HTMLInputElement {
  //         attribute DOMString align;
  //         attribute DOMString useMap;
};
