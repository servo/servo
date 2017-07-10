/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmliframeelement
[HTMLConstructor]
interface HTMLIFrameElement : HTMLElement {
  [CEReactions]
           attribute DOMString src;
  // [CEReactions]
  //         attribute DOMString srcdoc;

  // https://github.com/servo/servo/issues/14453
  // [CEReactions]
  // attribute DOMString name;

  [SameObject, PutForwards=value]
           readonly attribute DOMTokenList sandbox;
  // [CEReactions]
  //         attribute boolean seamless;
  [CEReactions]
           attribute boolean allowFullscreen;
  [CEReactions]
           attribute DOMString width;
  [CEReactions]
           attribute DOMString height;
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;

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

  // [CEReactions, TreatNullAs=EmptyString]
  // attribute DOMString marginHeight;
  // [CEReactions, TreatNullAs=EmptyString]
  // attribute DOMString marginWidth;
};

partial interface HTMLIFrameElement {
    [CEReactions, Func="::dom::window::Window::global_is_mozbrowser"]
    attribute boolean mozbrowser;

    [CEReactions, Func="::dom::window::Window::global_is_mozbrowser"]
    attribute boolean mozprivatebrowsing;
};

HTMLIFrameElement implements BrowserElement;
