/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#htmlformelement
 *
 * ⓒ Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[OverrideBuiltins]
interface HTMLFormElement : HTMLElement {
           [Pure, SetterThrows]
           attribute DOMString acceptCharset;
           [Pure, SetterThrows]
           attribute DOMString action;
           [Pure, SetterThrows]
           attribute DOMString autocomplete;
           [Pure, SetterThrows]
           attribute DOMString enctype;
           [Pure, SetterThrows]
           attribute DOMString encoding;
           [Pure, SetterThrows]
           attribute DOMString method;
           [Pure, SetterThrows]
           attribute DOMString name;
           [Pure, SetterThrows]
           attribute boolean noValidate;
           [Pure, SetterThrows]
           attribute DOMString target;

  [Constant]
  readonly attribute HTMLCollection elements;
  [Pure]
  readonly attribute long length;

  getter Element (unsigned long index);
  // TODO this should be: getter (RadioNodeList or HTMLInputElement or HTMLImageElement) (DOMString name);
//  getter nsISupports (DOMString name);

  [Throws]
  void submit();
  void reset();
  boolean checkValidity();
};
