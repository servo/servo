/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlselectelement
[HTMLConstructor]
interface HTMLSelectElement : HTMLElement {
  //         attribute boolean autofocus;
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
           attribute boolean multiple;
           attribute DOMString name;
  //         attribute boolean required;
           attribute unsigned long size;

  readonly attribute DOMString type;

  readonly attribute HTMLOptionsCollection options;
           attribute unsigned long length;
  getter Element? item(unsigned long index);
  HTMLOptionElement? namedItem(DOMString name);
  // Note: this function currently only exists for union.html.
  void add((HTMLOptionElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);
  void remove(); // ChildNode overload
  void remove(long index);
  //setter void (unsigned long index, HTMLOptionElement? option);

  //readonly attribute HTMLCollection selectedOptions;
  attribute long selectedIndex;
  attribute DOMString value;

  //readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  //readonly attribute DOMString validationMessage;
  //boolean checkValidity();
  //boolean reportValidity();
  //void setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
