/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/html/#the-select-element
 */

interface HTMLSelectElement : HTMLElement {
  [SetterThrows, Pure]
           attribute boolean autofocus;
  [SetterThrows, Pure]
           attribute boolean disabled;
  [Pure]
  readonly attribute HTMLFormElement? form;
  [SetterThrows, Pure]
           attribute boolean multiple;
  [SetterThrows, Pure]
           attribute DOMString name;
  [SetterThrows, Pure]
           attribute boolean required;
  [SetterThrows, Pure]
           attribute unsigned long size;

  [Pure]
  readonly attribute DOMString type;

  /*[Constant]
    readonly attribute HTMLOptionsCollection options;*/
  [SetterThrows, Pure]
           attribute unsigned long length;
  getter Element? item(unsigned long index);
  HTMLOptionElement? namedItem(DOMString name);
  /*[Throws]
    void add((HTMLOptionElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);*/
  void remove(long index);
  [Throws]
  setter creator void (unsigned long index, HTMLOptionElement? option);

// NYI:  readonly attribute HTMLCollection selectedOptions;
  [SetterThrows, Pure]
           attribute long selectedIndex;
  [Pure]
           attribute DOMString value;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  void setCustomValidity(DOMString error);

// NYI:  readonly attribute NodeList labels;

  // https://www.w3.org/Bugs/Public/show_bug.cgi?id=20720
  void remove();
};
