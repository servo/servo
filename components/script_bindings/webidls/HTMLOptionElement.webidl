/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmloptionelement
[Exposed=Window, LegacyFactoryFunction=Option(optional DOMString text = "", optional DOMString value,
                         optional boolean defaultSelected = false,
                         optional boolean selected = false)]
interface HTMLOptionElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  [CEReactions]
           attribute DOMString label;
  [CEReactions]
           attribute boolean defaultSelected;
           attribute boolean selected;
  [CEReactions]
           attribute DOMString value;

  [CEReactions]
           attribute DOMString text;
  readonly attribute long index;
};
