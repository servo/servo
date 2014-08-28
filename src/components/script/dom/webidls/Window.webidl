/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#window
[PrimaryGlobal]
/*sealed*/ interface Window : EventTarget {
  // the current browsing context
  //[Unforgeable] readonly attribute WindowProxy window;
  //[Replaceable] readonly attribute WindowProxy self;
  readonly attribute Window window;
  readonly attribute Window self;
  /*[Unforgeable]*/ readonly attribute Document document;
  //         attribute DOMString name;
  /*[PutForwards=href, Unforgeable]*/ readonly attribute Location location;
  //readonly attribute History history;
  //[Replaceable] readonly attribute BarProp locationbar;
  //[Replaceable] readonly attribute BarProp menubar;
  //[Replaceable] readonly attribute BarProp personalbar;
  //[Replaceable] readonly attribute BarProp scrollbars;
  //[Replaceable] readonly attribute BarProp statusbar;
  //[Replaceable] readonly attribute BarProp toolbar;
  //         attribute DOMString status;
  void close();
  //readonly attribute boolean closed;
  //void stop();
  //void focus();
  //void blur();

  // other browsing contexts
  //[Replaceable] readonly attribute WindowProxy frames;
  //[Replaceable] readonly attribute unsigned long length;
  //[Unforgeable] readonly attribute WindowProxy top;
  //         attribute any opener;
  //readonly attribute WindowProxy parent;
  readonly attribute Window parent;
  //readonly attribute Element? frameElement;
  //WindowProxy open(optional DOMString url = "about:blank", optional DOMString target = "_blank", optional DOMString features = "", optional boolean replace = false);
  //getter WindowProxy (unsigned long index);
  //getter object (DOMString name);

  // the user agent
  readonly attribute Navigator navigator;
  //[Replaceable] readonly attribute External external;
  //readonly attribute ApplicationCache applicationCache;

  // user prompts
  //void alert();
  void alert(DOMString message);
  //boolean confirm(optional DOMString message = "");
  //DOMString? prompt(optional DOMString message = "", optional DOMString default = "");
  //void print();
  //any showModalDialog(DOMString url, optional any argument);

  //void postMessage(any message, DOMString targetOrigin, optional sequence<Transferable> transfer);

  // also has obsolete members
};
Window implements GlobalEventHandlers;
Window implements WindowEventHandlers;

// http://www.whatwg.org/html/#windowtimers
[NoInterfaceObject/*, Exposed=Window,Worker*/]
interface WindowTimers {
  //long setTimeout(Function handler, optional long timeout = 0, any... arguments);
  //long setTimeout(DOMString handler, optional long timeout = 0, any... arguments);
  long setTimeout(any handler, optional long timeout = 0);
  void clearTimeout(optional long handle = 0);
  //long setInterval(Function handler, optional long timeout = 0, any... arguments);
  //long setInterval(DOMString handler, optional long timeout = 0, any... arguments);
  long setInterval(any handler, optional long timeout = 0);
  void clearInterval(optional long handle = 0);
};
Window implements WindowTimers;

// http://www.whatwg.org/html/#atob
[NoInterfaceObject/*, Exposed=Window,Worker*/]
interface WindowBase64 {
  [Throws]
  DOMString btoa(DOMString btoa);
  [Throws]
  DOMString atob(DOMString atob);
};
Window implements WindowBase64;

// https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#sec-window.performance-attribute
partial interface Window {
  /*[Replaceable]*/ readonly attribute Performance performance;
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-window-interface
partial interface Window {
  //MediaQueryList matchMedia(DOMString query);
  [SameObject] readonly attribute Screen screen;

  // browsing context
  //void moveTo(double x, double y);
  //void moveBy(double x, double y);
  //void resizeTo(double x, double y);
  //void resizeBy(double x, double y);

  // viewport
  //readonly attribute double innerWidth;
  //readonly attribute double innerHeight;

  // viewport scrolling
  //readonly attribute double scrollX;
  //readonly attribute double pageXOffset;
  //readonly attribute double scrollY;
  //readonly attribute double pageYOffset;
  //void scroll(double x, double y, optional ScrollOptions options);
  //void scrollTo(double x, double y, optional ScrollOptions options);
  //void scrollBy(double x, double y, optional ScrollOptions options);

  // client
  //readonly attribute double screenX;
  //readonly attribute double screenY;
  //readonly attribute double outerWidth;
  //readonly attribute double outerHeight;
  //readonly attribute double devicePixelRatio;
};

// Proprietary extensions.
partial interface Window {
  readonly attribute Console console;
  void debug(DOMString arg);
  void gc();
};
Window implements OnErrorEventHandlerForWindow;
