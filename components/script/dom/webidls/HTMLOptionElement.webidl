/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#htmloptionelement
//[NamedConstructor=Option(optional DOMString text = "", optional DOMString value, optional boolean defaultSelected = false, optional boolean selected = false)]
interface HTMLOptionElement : HTMLElement {
           attribute boolean disabled;
  //readonly attribute HTMLFormElement? form;
  //         attribute DOMString label;
  //         attribute boolean defaultSelected;
  //         attribute boolean selected;
  //         attribute DOMString value;

             attribute DOMString text;
  //readonly attribute long index;
};
