/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlformelement
[/*OverrideBuiltins, */HTMLConstructor]
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

  [SameObject] readonly attribute HTMLFormControlsCollection elements;
  readonly attribute unsigned long length;
  getter Element? (unsigned long index);
  //getter (RadioNodeList or Element) (DOMString name);

  void submit();
  void reset();
  //boolean checkValidity();
  //boolean reportValidity();
};
