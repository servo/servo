/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#window
[Global=Window, Exposed=Window /*, LegacyUnenumerableNamedProperties */]
/*sealed*/ interface Window : GlobalScope {
  // the current browsing context
  [Unforgeable] readonly attribute WindowProxy window;
  [BinaryName="Self_", Replaceable] readonly attribute WindowProxy self;
  [Unforgeable] readonly attribute Document document;

  attribute DOMString name;

  [PutForwards=href, Unforgeable] readonly attribute Location location;
  readonly attribute History history;
  [Pref="dom.customelements.enabled"]
  readonly attribute CustomElementRegistry customElements;
  //[Replaceable] readonly attribute BarProp locationbar;
  //[Replaceable] readonly attribute BarProp menubar;
  //[Replaceable] readonly attribute BarProp personalbar;
  //[Replaceable] readonly attribute BarProp scrollbars;
  //[Replaceable] readonly attribute BarProp statusbar;
  //[Replaceable] readonly attribute BarProp toolbar;
  attribute DOMString status;
  void close();
  readonly attribute boolean closed;
  void stop();
  //void focus();
  //void blur();

  // other browsing contexts
  [Replaceable] readonly attribute WindowProxy frames;
  [Replaceable] readonly attribute unsigned long length;
  // Note that this can return null in the case that the browsing context has been discarded.
  // https://github.com/whatwg/html/issues/2115
  [Unforgeable] readonly attribute WindowProxy? top;
  attribute any opener;
  // Note that this can return null in the case that the browsing context has been discarded.
  // https://github.com/whatwg/html/issues/2115
  [Replaceable] readonly attribute WindowProxy? parent;
  readonly attribute Element? frameElement;
  WindowProxy? open(optional USVString url = "", optional DOMString target = "_blank",
                    optional DOMString features = "");
  //getter WindowProxy (unsigned long index);

  // https://github.com/servo/servo/issues/14453
  // getter object (DOMString name);

  // the user agent
  readonly attribute Navigator navigator;
  //[Replaceable] readonly attribute External external;
  //readonly attribute ApplicationCache applicationCache;

  // user prompts
  void alert(DOMString message);
  void alert();
  boolean confirm(optional DOMString message = "");
  DOMString? prompt(optional DOMString message = "", optional DOMString default = "");
  //void print();
  //any showModalDialog(DOMString url, optional any argument);

  unsigned long requestAnimationFrame(FrameRequestCallback callback);
  void cancelAnimationFrame(unsigned long handle);

  [Throws]
  void postMessage(any message, USVString targetOrigin, optional sequence<object> transfer = []);
  [Throws]
  void postMessage(any message, optional WindowPostMessageOptions options = {});

  // also has obsolete members
};
Window includes GlobalEventHandlers;
Window includes WindowEventHandlers;

// https://html.spec.whatwg.org/multipage/#Window-partial
partial interface Window {
  void captureEvents();
  void releaseEvents();
};

// https://drafts.csswg.org/cssom/#extensions-to-the-window-interface
partial interface Window {
   [NewObject]
   CSSStyleDeclaration getComputedStyle(Element elt, optional DOMString pseudoElt);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
enum ScrollBehavior { "auto", "instant", "smooth" };

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
dictionary ScrollOptions {
    ScrollBehavior behavior = "auto";
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
dictionary ScrollToOptions : ScrollOptions {
    unrestricted double left;
    unrestricted double top;
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
partial interface Window {
  [Exposed=(Window), NewObject] MediaQueryList matchMedia(DOMString query);
  [SameObject, Replaceable] readonly attribute Screen screen;

  // browsing context
  void moveTo(long x, long y);
  void moveBy(long x, long y);
  void resizeTo(long x, long y);
  void resizeBy(long x, long y);

  // viewport
  [Replaceable] readonly attribute long innerWidth;
  [Replaceable] readonly attribute long innerHeight;

  // viewport scrolling
  [Replaceable] readonly attribute long scrollX;
  [Replaceable] readonly attribute long pageXOffset;
  [Replaceable] readonly attribute long scrollY;
  [Replaceable] readonly attribute long pageYOffset;
  void scroll(optional ScrollToOptions options = {});
  void scroll(unrestricted double x, unrestricted double y);
  void scrollTo(optional ScrollToOptions options = {});
  void scrollTo(unrestricted double x, unrestricted double y);
  void scrollBy(optional ScrollToOptions options = {});
  void scrollBy(unrestricted double x, unrestricted double y);

  // client
  [Replaceable] readonly attribute long screenX;
  [Replaceable] readonly attribute long screenY;
  [Replaceable] readonly attribute long outerWidth;
  [Replaceable] readonly attribute long outerHeight;
  [Replaceable] readonly attribute double devicePixelRatio;
};

// Proprietary extensions.
partial interface Window {
  void debug(DOMString arg);
  void gc();
  void trap();
};

// WebDriver extensions
partial interface Window {
  // Shouldn't be public, but just to make things work for now
  void webdriverCallback(optional any result);
  void webdriverTimeout();
};

// https://html.spec.whatwg.org/multipage/#dom-sessionstorage
interface mixin WindowSessionStorage {
  readonly attribute Storage sessionStorage;
};
Window includes WindowSessionStorage;

// https://html.spec.whatwg.org/multipage/#dom-localstorage
interface mixin WindowLocalStorage {
  readonly attribute Storage localStorage;
};
Window includes WindowLocalStorage;

// http://w3c.github.io/animation-timing/#framerequestcallback
callback FrameRequestCallback = void (DOMHighResTimeStamp time);

// https://webbluetoothcg.github.io/web-bluetooth/tests#test-interfaces
partial interface Window {
   [Pref="dom.bluetooth.testing.enabled", Exposed=Window]
   readonly attribute TestRunner testRunner;
   //readonly attribute EventSender eventSender;
};

partial interface Window {
   [Pref="css.animations.testing.enabled"]
   readonly attribute unsigned long runningAnimationCount;
};

dictionary WindowPostMessageOptions : PostMessageOptions {
   USVString targetOrigin = "/";
};
