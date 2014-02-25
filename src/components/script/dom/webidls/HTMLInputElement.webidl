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
  [Pure, SetterThrows]
           attribute boolean disabled;
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
           attribute unsigned long width;

  [Pure]
  readonly attribute boolean willValidate;
  [GetterThrows]
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);

  void select();

  [Throws]
           // TODO: unsigned vs signed
           attribute long selectionStart;
  [Throws]
           attribute long selectionEnd;
  [Throws]
           attribute DOMString selectionDirection;
  // also has obsolete members
};

partial interface HTMLInputElement {
  [Pure, SetterThrows]
           attribute DOMString align;
  [Pure, SetterThrows]
           attribute DOMString useMap;
};
