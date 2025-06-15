/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlelement
[Exposed=Window]
interface HTMLElement : Element {
  [HTMLConstructor] constructor();

  // metadata attributes
  [CEReactions]
           attribute DOMString title;
  [CEReactions]
           attribute DOMString lang;
  [CEReactions]
           attribute boolean translate;
  [CEReactions]
           attribute DOMString dir;
  readonly attribute DOMStringMap dataset;

  // microdata
  //         attribute boolean itemScope;

  //         attribute DOMString itemId;
  //readonly attribute HTMLPropertiesCollection properties;
  //         attribute any itemValue; // acts as DOMString on setting
  [Pref="dom_microdata_testing_enabled"]
  sequence<DOMString>? propertyNames();
  [Pref="dom_microdata_testing_enabled"]
  sequence<DOMString>? itemtypes();

  // user interaction
  [CEReactions]
           attribute boolean hidden;
  undefined click();
  // [CEReactions]
  //         attribute long tabIndex;
  undefined focus();
  undefined blur();
  // [CEReactions]
  //         attribute DOMString accessKey;
  //readonly attribute DOMString accessKeyLabel;
  // [CEReactions]
  //         attribute boolean draggable;
  // [SameObject, PutForwards=value] readonly attribute DOMTokenList dropzone;
  //         attribute HTMLMenuElement? contextMenu;
  [Unimplemented, CEReactions] attribute boolean spellcheck;
  [Unimplemented] undefined forceSpellCheck();

  [CEReactions] attribute [LegacyNullToEmptyString] DOMString innerText;
  [CEReactions, Throws] attribute [LegacyNullToEmptyString] DOMString outerText;

  [Throws] ElementInternals attachInternals();

  // command API
  // readonly attribute DOMString? commandType;
  // readonly attribute DOMString? commandLabel;
  // readonly attribute DOMString? commandIcon;
  // readonly attribute boolean? commandHidden;
  // readonly attribute boolean? commandDisabled;
  // readonly attribute boolean? commandChecked;
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-htmlelement-interface
partial interface HTMLElement {
  // CSSOM things are not [Pure] because they can flush
  readonly attribute Element? offsetParent;
  readonly attribute long offsetTop;
  readonly attribute long offsetLeft;
  readonly attribute long offsetWidth;
  readonly attribute long offsetHeight;
};

HTMLElement includes GlobalEventHandlers;
HTMLElement includes ElementContentEditable;
HTMLElement includes ElementCSSInlineStyle;
HTMLElement includes HTMLOrSVGElement;
