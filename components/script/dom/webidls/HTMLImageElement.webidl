/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlimageelement
[HTMLConstructor, NamedConstructor=Image(optional unsigned long width, optional unsigned long height)]
interface HTMLImageElement : HTMLElement {
  [CEReactions]
           attribute DOMString alt;
  [CEReactions]
           attribute DOMString src;
  // [CEReactions]
  //         attribute DOMString srcset;
  [CEReactions]
           attribute DOMString? crossOrigin;
  [CEReactions]
           attribute DOMString useMap;
  [CEReactions]
           attribute boolean isMap;
  [CEReactions]
           attribute unsigned long width;
  [CEReactions]
           attribute unsigned long height;
  readonly attribute unsigned long naturalWidth;
  readonly attribute unsigned long naturalHeight;
  readonly attribute boolean complete;
  readonly attribute DOMString currentSrc;
  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLImageElement-partial
partial interface HTMLImageElement {
  [CEReactions]
           attribute DOMString name;
  // [CEReactions]
  //          attribute DOMString lowsrc;
  [CEReactions]
           attribute DOMString align;
  [CEReactions]
           attribute unsigned long hspace;
  [CEReactions]
           attribute unsigned long vspace;
  [CEReactions]
           attribute DOMString longDesc;

  [CEReactions, TreatNullAs=EmptyString]
  attribute DOMString border;
};

// https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlimageelement-interface
partial interface HTMLImageElement {
  // readonly attribute long x;
  // readonly attribute long y;
};
