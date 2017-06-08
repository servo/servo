/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmloptionelement
[HTMLConstructor/*, NamedConstructor=Option(optional DOMString text = "", optional DOMString value,
                         optional boolean defaultSelected = false,
                         optional boolean selected = false)*/]
interface HTMLOptionElement : HTMLElement {
             attribute boolean disabled;
             readonly attribute HTMLFormElement? form;
             attribute DOMString label;
             attribute boolean defaultSelected;
             attribute boolean selected;
             attribute DOMString value;

             attribute DOMString text;
  //readonly attribute long index;
};
