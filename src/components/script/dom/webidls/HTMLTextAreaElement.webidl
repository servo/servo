/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-textarea-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLTextAreaElement : HTMLElement {
  [SetterThrows, Pure]
           attribute boolean autofocus;
  [SetterThrows, Pure]
           attribute unsigned long cols;
  [SetterThrows, Pure]
           attribute boolean disabled;
  [SetterThrows, Pure]
           attribute long maxLength;
  [SetterThrows, Pure]
           attribute DOMString name;
  [SetterThrows, Pure]
           attribute DOMString placeholder;
  [SetterThrows, Pure]
           attribute boolean readOnly;
  [SetterThrows, Pure]
           attribute boolean required;
  [SetterThrows, Pure]
           attribute unsigned long rows;
  [SetterThrows, Pure]
           attribute DOMString wrap;

  [Constant]
  readonly attribute DOMString type;
  [SetterThrows, Pure]
           attribute DOMString defaultValue;
  [TreatNullAs=EmptyString] attribute DOMString value;
  readonly attribute unsigned long textLength;

  readonly attribute boolean willValidate;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);
  void select();
  [Throws]
           attribute unsigned long selectionStart;
  [Throws]
           attribute unsigned long selectionEnd;
  [Throws]
           attribute DOMString selectionDirection;
  void setRangeText(DOMString replacement);
};
