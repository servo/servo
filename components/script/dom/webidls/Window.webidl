/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#window
[Global=Window, Exposed=Window /*, LegacyUnenumerableNamedProperties */]
/*sealed*/ interface Window : GlobalScope {
  // the current browsing context
  [LegacyUnforgeable, CrossOriginReadable] readonly attribute WindowProxy window;
  [BinaryName="Self_", Replaceable, CrossOriginReadable] readonly attribute WindowProxy self;
  [LegacyUnforgeable] readonly attribute Document document;

  attribute DOMString name;

  [PutForwards=href, LegacyUnforgeable, CrossOriginReadable, CrossOriginWritable]
    readonly attribute Location location;
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
  [CrossOriginCallable] undefined close();
  [CrossOriginReadable] readonly attribute boolean closed;
  undefined stop();
  //[CrossOriginCallable] void focus();
  //[CrossOriginCallable] void blur();

  // other browsing contexts
  [Replaceable, CrossOriginReadable] readonly attribute WindowProxy frames;
  [Replaceable, CrossOriginReadable] readonly attribute unsigned long length;
  // Note that this can return null in the case that the browsing context has been discarded.
  // https://github.com/whatwg/html/issues/2115
  [LegacyUnforgeable, CrossOriginReadable] readonly attribute WindowProxy? top;
  [CrossOriginReadable] attribute any opener;
  // Note that this can return null in the case that the browsing context has been discarded.
  // https://github.com/whatwg/html/issues/2115
  [Replaceable, CrossOriginReadable] readonly attribute WindowProxy? parent;
  readonly attribute Element? frameElement;
  [Throws] WindowProxy? open(optional USVString url = "", optional DOMString target = "_blank",
                             optional DOMString features = "");
  //getter WindowProxy (unsigned long index);

  getter object (DOMString name);

  // the user agent
  readonly attribute Navigator navigator;
  //[Replaceable] readonly attribute External external;
  //readonly attribute ApplicationCache applicationCache;

  // user prompts
  undefined alert(DOMString message);
  undefined alert();
  boolean confirm(optional DOMString message = "");
  DOMString? prompt(optional DOMString message = "", optional DOMString default = "");
  //void print();
  //any showModalDialog(DOMString url, optional any argument);

  unsigned long requestAnimationFrame(FrameRequestCallback callback);
  undefined cancelAnimationFrame(unsigned long handle);

  [Throws, CrossOriginCallable]
  undefined postMessage(any message, USVString targetOrigin, optional sequence<object> transfer = []);
  [Throws, CrossOriginCallable]
  undefined postMessage(any message, optional WindowPostMessageOptions options = {});

  // also has obsolete members
};
Window includes GlobalEventHandlers;
Window includes WindowEventHandlers;

// https://html.spec.whatwg.org/multipage/#Window-partial
partial interface Window {
  undefined captureEvents();
  undefined releaseEvents();
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
  undefined moveTo(long x, long y);
  undefined moveBy(long x, long y);
  undefined resizeTo(long x, long y);
  undefined resizeBy(long x, long y);

  // viewport
  [Replaceable] readonly attribute long innerWidth;
  [Replaceable] readonly attribute long innerHeight;

  // viewport scrolling
  [Replaceable] readonly attribute long scrollX;
  [Replaceable] readonly attribute long pageXOffset;
  [Replaceable] readonly attribute long scrollY;
  [Replaceable] readonly attribute long pageYOffset;
  undefined scroll(optional ScrollToOptions options = {});
  undefined scroll(unrestricted double x, unrestricted double y);
  undefined scrollTo(optional ScrollToOptions options = {});
  undefined scrollTo(unrestricted double x, unrestricted double y);
  undefined scrollBy(optional ScrollToOptions options = {});
  undefined scrollBy(unrestricted double x, unrestricted double y);

  // client
  [Replaceable] readonly attribute long screenX;
  [Replaceable] readonly attribute long screenY;
  [Replaceable] readonly attribute long outerWidth;
  [Replaceable] readonly attribute long outerHeight;
  [Replaceable] readonly attribute double devicePixelRatio;
};

// Proprietary extensions.
partial interface Window {
  [Pref="dom.servo_helpers.enabled"]
  undefined debug(DOMString arg);
  [Pref="dom.servo_helpers.enabled"]
  undefined gc();
  [Pref="dom.servo_helpers.enabled"]
  undefined js_backtrace();
};

// WebDriver extensions
partial interface Window {
  // Shouldn't be public, but just to make things work for now
  undefined webdriverCallback(optional any result);
  undefined webdriverTimeout();
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
callback FrameRequestCallback = undefined (DOMHighResTimeStamp time);

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

// https://w3c.github.io/selection-api/#dom-document
partial interface Window {
   Selection? getSelection();
};

// https://dom.spec.whatwg.org/#interface-window-extensions
partial interface Window {
  [Replaceable] readonly attribute any event; // historical
};

dictionary WindowPostMessageOptions : PostMessageOptions {
   USVString targetOrigin = "/";
};
