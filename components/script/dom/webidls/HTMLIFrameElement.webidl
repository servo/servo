/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmliframeelement
interface HTMLIFrameElement : HTMLElement {
           attribute DOMString src;
  //         attribute DOMString srcdoc;
  //         attribute DOMString name;
  //[PutForwards=value] readonly attribute DOMSettableTokenList sandbox;
           attribute DOMString sandbox;
  //         attribute boolean seamless;
  //         attribute boolean allowFullscreen;
           attribute DOMString width;
           attribute DOMString height;
  readonly attribute Document? contentDocument;
  //readonly attribute WindowProxy? contentWindow;
  readonly attribute Window? contentWindow;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLIFrameElement-partial
partial interface HTMLIFrameElement {
  //         attribute DOMString align;
  //         attribute DOMString scrolling;
  //         attribute DOMString frameBorder;
  //         attribute DOMString longDesc;

  //[TreatNullAs=EmptyString] attribute DOMString marginHeight;
  //[TreatNullAs=EmptyString] attribute DOMString marginWidth;
};

partial interface HTMLIFrameElement {
  [ChromeOnly,SetterThrows]
           attribute boolean mozbrowser;
};

HTMLIFrameElement implements BrowserElement;
