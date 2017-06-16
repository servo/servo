/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmliframeelement
[HTMLConstructor]
interface HTMLIFrameElement : HTMLElement {
           attribute DOMString src;
  //         attribute DOMString srcdoc;

  // https://github.com/servo/servo/issues/14453
  // attribute DOMString name;

           [SameObject, PutForwards=value]
           readonly attribute DOMTokenList sandbox;
  //         attribute boolean seamless;
           attribute boolean allowFullscreen;
           attribute DOMString width;
           attribute DOMString height;
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;

  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLIFrameElement-partial
partial interface HTMLIFrameElement {
  //         attribute DOMString align;
  //         attribute DOMString scrolling;
           attribute DOMString frameBorder;
  //         attribute DOMString longDesc;

  //[TreatNullAs=EmptyString] attribute DOMString marginHeight;
  //[TreatNullAs=EmptyString] attribute DOMString marginWidth;
};

partial interface HTMLIFrameElement {
    [Func="::dom::window::Window::global_is_mozbrowser"]
    attribute boolean mozbrowser;

    [Func="::dom::window::Window::global_is_mozbrowser"]
    attribute boolean mozprivatebrowsing;
};

HTMLIFrameElement implements BrowserElement;
