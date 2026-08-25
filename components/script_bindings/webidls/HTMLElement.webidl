/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlelement
[Exposed=Window]
interface HTMLElement : Element {
  [HTMLConstructor] constructor();

  // metadata attributes
  [CEReactions] attribute DOMString title;
  [CEReactions] attribute DOMString lang;
  [CEReactions] attribute boolean translate;
  [CEReactions] attribute DOMString dir;
  readonly attribute DOMStringMap dataset;

  [Pref="dom_microdata_testing_enabled"]
  sequence<DOMString>? propertyNames();
  [Pref="dom_microdata_testing_enabled"]
  sequence<DOMString>? itemtypes();

  // user interaction
  [CEReactions] attribute boolean hidden;
  // [CEReactions, Reflect] attribute boolean inert;
  undefined click();
  [CEReactions] attribute DOMString accessKey;
  readonly attribute DOMString accessKeyLabel;
  // [CEReactions] attribute boolean draggable;
  // [CEReactions] attribute boolean spellcheck;
  // [CEReactions, ReflectSetter] attribute DOMString writingSuggestions;
  // [CEReactions, ReflectSetter] attribute DOMString autocapitalize;
  // [CEReactions] attribute boolean autocorrect;

  [CEReactions] attribute [LegacyNullToEmptyString] DOMString innerText;
  [CEReactions, Throws] attribute [LegacyNullToEmptyString] DOMString outerText;

  [Throws] ElementInternals attachInternals();

  // The popover API
  // undefined showPopover(optional ShowPopoverOptions options = {});
  // undefined hidePopover();
  // boolean togglePopover(optional (TogglePopoverOptions or boolean) options = {});
  // [CEReactions] attribute DOMString? popover;

  // [CEReactions, Reflect, ReflectRange=(0, 8)] attribute unsigned long headingOffset;
  // [CEReactions, Reflect] attribute boolean headingReset;
};

// dictionary ShowPopoverOptions {
//   HTMLElement source;
// };
//
// dictionary TogglePopoverOptions : ShowPopoverOptions {
//   boolean force;
// };

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-htmlelement-interface
partial interface HTMLElement {
  // CSSOM things are not [Pure] because they can flush
  Element? scrollParent();
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
