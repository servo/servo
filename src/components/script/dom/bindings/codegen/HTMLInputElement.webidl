/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-input-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// original import from:
// http://hg.mozilla.org/mozilla-central/filelog/8c240c67f76c/dom/webidl/HTMLInputElement.webidl

/*
interface nsIControllers;
*/

interface HTMLInputElement : HTMLElement {
  [Pure, SetterThrows]
           attribute DOMString accept;
  [Pure, SetterThrows]
           attribute DOMString alt;
  [Pure, SetterThrows]
           attribute DOMString autocomplete;
  [Pure, SetterThrows]
           attribute boolean autofocus;
  [Pure, SetterThrows]
           attribute boolean defaultChecked;
  [Pure]
           attribute boolean checked;
/*
           attribute DOMString dirName;
*/
  [Pure, SetterThrows]
           attribute boolean disabled;
/*
  readonly attribute HTMLFormElement? form;
  [Pure]
  readonly attribute FileList? files;
*/
  [Pure, SetterThrows]
           attribute DOMString formAction;
  [Pure, SetterThrows]
           attribute DOMString formEnctype;
  [Pure, SetterThrows]
           attribute DOMString formMethod;
  [Pure, SetterThrows]
           attribute boolean formNoValidate;
  [Pure, SetterThrows]
           attribute DOMString formTarget;
  [Pure, SetterThrows]
           attribute unsigned long height;
  [Pure]
           attribute boolean indeterminate;
  [Pure, SetterThrows]
           attribute DOMString inputMode;
/*
  [Pure]
  readonly attribute HTMLElement? list;
*/
  [Pure, SetterThrows]
           attribute DOMString max;
  [Pure, SetterThrows]
           attribute long maxLength;
  [Pure, SetterThrows]
           attribute DOMString min;
  [Pure, SetterThrows]
           attribute boolean multiple;
  [Pure, SetterThrows]
           attribute DOMString name;
  [Pure, SetterThrows]
           attribute DOMString pattern;
  [Pure, SetterThrows]
           attribute DOMString placeholder;
  [Pure, SetterThrows]
           attribute boolean readOnly;
  [Pure, SetterThrows]
           attribute boolean required;
  [Pure, SetterThrows]
           attribute unsigned long size;
  [Pure, SetterThrows]
           attribute DOMString src;
  [Pure, SetterThrows]
           attribute DOMString step;
  [Pure, SetterThrows]
           attribute DOMString type;
  [Pure, SetterThrows]
           attribute DOMString defaultValue;
  [Pure, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString value;
/*
  [Throws, Pref="dom.experimental_forms"]
           attribute Date? valueAsDate;
  [Pure, SetterThrows]
           attribute unrestricted double valueAsNumber;
*/
           attribute unsigned long width;
/*
  [Throws]
  void stepUp(optional long n = 1);
  [Throws]
  void stepDown(optional long n = 1);
*/

  [Pure]
  readonly attribute boolean willValidate;
/*
  [Pure]
  readonly attribute ValidityState validity;
*/
  [GetterThrows]
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);

/*
  readonly attribute NodeList labels;
*/

  void select();

  [Throws]
           // TODO: unsigned vs signed
           attribute long selectionStart;
  [Throws]
           attribute long selectionEnd;
  [Throws]
           attribute DOMString selectionDirection;
/*
  // Bug 850364 void setRangeText(DOMString replacement);
  // Bug 850364 setRangeText(DOMString replacement, unsigned long start, unsigned long end, optional SelectionMode selectionMode);
*/
  // also has obsolete members
};

partial interface HTMLInputElement {
  [Pure, SetterThrows]
           attribute DOMString align;
  [Pure, SetterThrows]
           attribute DOMString useMap;
};

/*
// Mozilla extensions
partial interface HTMLInputElement {
  [Throws]
  void setSelectionRange(long start, long end, optional DOMString direction);

  [GetterThrows]
  readonly attribute nsIControllers        controllers;
  [GetterThrows]
  readonly attribute long                  textLength;

  [ChromeOnly]
  sequence<DOMString> mozGetFileNameArray();

  [ChromeOnly]
  void mozSetFileNameArray(sequence<DOMString> fileNames);

  boolean mozIsTextField(boolean aExcludePassword);
};

partial interface HTMLInputElement {
  // Mirrored chrome-only nsIDOMNSEditableElement methods.  Please make sure
  // to update this list if nsIDOMNSEditableElement changes.

  [Pure, ChromeOnly]
  readonly attribute nsIEditor? editor;

  // This is similar to set .value on nsIDOMInput/TextAreaElements, but handling
  // of the value change is closer to the normal user input, so 'change' event
  // for example will be dispatched when focusing out the element.
  [ChromeOnly]
  void setUserInput(DOMString input);
};

[NoInterfaceObject]
interface MozPhonetic {
  [Pure, ChromeOnly]
  readonly attribute DOMString phonetic;
};

HTMLInputElement implements MozImageLoadingContent;
HTMLInputElement implements MozPhonetic;
*/
