/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlformelement
[Exposed=Window]
interface HTMLFormElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute DOMString acceptCharset;
  [CEReactions]
           attribute DOMString action;
  [CEReactions]
           attribute DOMString autocomplete;
  [CEReactions]
           attribute DOMString enctype;
  [CEReactions]
           attribute DOMString encoding;
  [CEReactions]
           attribute DOMString method;
  [CEReactions]
           attribute DOMString name;
  [CEReactions]
           attribute boolean noValidate;
  [CEReactions]
           attribute DOMString target;

  [SameObject] readonly attribute HTMLFormControlsCollection elements;
  readonly attribute unsigned long length;
  getter Element? (unsigned long index);
  getter (RadioNodeList or Element) (DOMString name);

  void submit();
  [CEReactions]
  void reset();
  //boolean checkValidity();
  //boolean reportValidity();
};

// https://html.spec.whatwg.org/multipage/#selectionmode
enum SelectionMode {
  "preserve", // default
  "select",
  "start",
  "end"
};
