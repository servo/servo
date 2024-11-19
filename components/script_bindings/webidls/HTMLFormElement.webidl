/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlformelement
[Exposed=Window, LegacyUnenumerableNamedProperties]
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
  [CEReactions]
           attribute DOMString rel;
  [SameObject, PutForwards=value] readonly attribute DOMTokenList relList;

  [SameObject] readonly attribute HTMLFormControlsCollection elements;
  readonly attribute unsigned long length;
  getter Element? (unsigned long index);
  getter (RadioNodeList or Element) (DOMString name);

  undefined submit();
  [Throws] undefined requestSubmit(optional HTMLElement? submitter = null);
  [CEReactions]
  undefined reset();
  boolean checkValidity();
  boolean reportValidity();
};

// https://html.spec.whatwg.org/multipage/#selectionmode
enum SelectionMode {
  "preserve", // default
  "select",
  "start",
  "end"
};
