/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlselectelement
[Exposed=Window]
interface HTMLSelectElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions]
  //          attribute boolean autofocus;
  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  [CEReactions]
           attribute boolean multiple;
  [CEReactions]
           attribute DOMString name;
  [CEReactions]
           attribute boolean required;
  [CEReactions]
           attribute unsigned long size;

  readonly attribute DOMString type;

  readonly attribute HTMLOptionsCollection options;
  [CEReactions]
           attribute unsigned long length;
  getter Element? item(unsigned long index);
  HTMLOptionElement? namedItem(DOMString name);

  [CEReactions, Throws]
  undefined add((HTMLOptionElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);
  [CEReactions]
  undefined remove(); // ChildNode overload
  [CEReactions]
  undefined remove(long index);
  [CEReactions, Throws] setter undefined (unsigned long index, HTMLOptionElement? option);

  // readonly attribute HTMLCollection selectedOptions;
  attribute long selectedIndex;
  attribute DOMString value;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;
};
