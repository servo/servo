/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlinputelement
[HTMLConstructor]
interface HTMLInputElement : HTMLElement {
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
  readonly attribute FileList? files;
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
  // readonly attribute HTMLElement? list;
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
           attribute DOMString src;
  [CEReactions]
           attribute DOMString step;
  [CEReactions]
           attribute DOMString type;
  [CEReactions]
           attribute DOMString defaultValue;
  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString value;
  //          attribute Date? valueAsDate;
  //          attribute unrestricted double valueAsNumber;
  //          attribute double valueLow;
  //          attribute double valueHigh;
  // [CEReactions]
  //          attribute unsigned long width;

  //void stepUp(optional long n = 1);
  //void stepDown(optional long n = 1);

  //readonly attribute boolean willValidate;
  //readonly attribute ValidityState validity;
  //readonly attribute DOMString validationMessage;
  //boolean checkValidity();
  //boolean reportValidity();
  //void setCustomValidity(DOMString error);

  readonly attribute NodeList labels;

  //void select();
  [SetterThrows]
           attribute unsigned long? selectionStart;
  [SetterThrows]
           attribute unsigned long? selectionEnd;
  [SetterThrows]
           attribute DOMString? selectionDirection;
  //void setRangeText(DOMString replacement);
  //void setRangeText(DOMString replacement, unsigned long start, unsigned long end,
  //                  optional SelectionMode selectionMode = "preserve");
  [Throws]
           void setSelectionRange(unsigned long start, unsigned long end, optional DOMString direction);

  // also has obsolete members

  // Select with file-system paths for testing purpose
  [Pref="dom.testing.htmlinputelement.select_files.enabled"]
  void selectFiles(sequence<DOMString> path);

};

// https://html.spec.whatwg.org/multipage/#HTMLInputElement-partial
partial interface HTMLInputElement {
  //         attribute DOMString align;
  //         attribute DOMString useMap;
};
