/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmlformelement
//[OverrideBuiltins]
interface HTMLFormElement : HTMLElement {
           attribute DOMString acceptCharset;
           attribute DOMString action;
           attribute DOMString autocomplete;
           attribute DOMString enctype;
           attribute DOMString encoding;
           attribute DOMString method;
           attribute DOMString name;
           attribute boolean noValidate;
           attribute DOMString target;

  //readonly attribute HTMLFormControlsCollection elements;
  //readonly attribute long length;
  //getter Element (unsigned long index);
  //getter (RadioNodeList or Element) (DOMString name);

  void submit();
  //void reset();
  //boolean checkValidity();
  //boolean reportValidity();

  //void requestAutocomplete();
};
