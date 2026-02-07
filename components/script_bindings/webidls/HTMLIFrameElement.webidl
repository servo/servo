/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmliframeelement
[Exposed=Window]
interface HTMLIFrameElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute USVString src;
  [CEReactions, SetterThrows] attribute (TrustedHTML or DOMString) srcdoc;
  [CEReactions] attribute DOMString name;
  [SameObject, PutForwards=value] readonly attribute DOMTokenList sandbox;
  // [CEReactions, Reflect] attribute DOMString allow;
  [CEReactions] attribute boolean allowFullscreen;
  [CEReactions] attribute DOMString width;
  [CEReactions] attribute DOMString height;
  [CEReactions] attribute DOMString referrerPolicy;
  [CEReactions] attribute DOMString loading;
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;
  // Document? getSVGDocument();

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLIFrameElement-partial
partial interface HTMLIFrameElement {
  // [CEReactions]
  //         attribute DOMString align;
  // [CEReactions]
  //         attribute DOMString scrolling;
  [CEReactions]
           attribute DOMString frameBorder;
  // [CEReactions]
  //         attribute DOMString longDesc;

  // [CEReactions, LegacyNullToEmptyString]
  // attribute DOMString marginHeight;
  // [CEReactions, LegacyNullToEmptyString]
  // attribute DOMString marginWidth;
};
