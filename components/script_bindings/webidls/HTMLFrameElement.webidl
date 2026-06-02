/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlframeelement
[Exposed=Window]
interface HTMLFrameElement : HTMLElement {
  [HTMLConstructor] constructor();

  // [CEReactions]
  //          attribute DOMString name;
  // [CEReactions]
  //          attribute DOMString scrolling;
  // [CEReactions]
  //          attribute DOMString src;
  // [CEReactions]
  //          attribute DOMString frameBorder;
  // [CEReactions]
  //          attribute DOMString longDesc;
  // [CEReactions]
  //          attribute boolean noResize;
  // readonly attribute Document? contentDocument;
  // readonly attribute WindowProxy? contentWindow;

  // [CEReactions, LegacyNullToEmptyString]
  // attribute DOMString marginHeight;
  // [CEReactions, LegacyNullToEmptyString]
  // attribute DOMString marginWidth;
};
