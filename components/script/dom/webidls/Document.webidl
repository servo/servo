/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-document
 * https://html.spec.whatwg.org/multipage/#the-document-object
 */

// https://dom.spec.whatwg.org/#interface-document
[Constructor]
interface Document : Node {
  [SameObject]
  readonly attribute DOMImplementation implementation;
  [Constant]
  readonly attribute USVString URL;
  [Constant]
  readonly attribute USVString documentURI;
  // readonly attribute USVString origin;
  readonly attribute DOMString compatMode;
  readonly attribute DOMString characterSet;
  readonly attribute DOMString charset; // legacy alias of .characterSet
  readonly attribute DOMString inputEncoding; // legacy alias of .characterSet
  [Constant]
  readonly attribute DOMString contentType;

  [Pure]
  readonly attribute DocumentType? doctype;
  [Pure]
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString qualifiedName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString qualifiedName);
  HTMLCollection getElementsByClassName(DOMString classNames);

  [NewObject, Throws]
  Element createElement(DOMString localName, optional ElementCreationOptions options);
  [NewObject, Throws]
  Element createElementNS(DOMString? namespace, DOMString qualifiedName, optional ElementCreationOptions options);
  [NewObject]
  DocumentFragment createDocumentFragment();
  [NewObject]
  Text createTextNode(DOMString data);
  [NewObject]
  Comment createComment(DOMString data);
  [NewObject, Throws]
  ProcessingInstruction createProcessingInstruction(DOMString target, DOMString data);

  [NewObject, Throws]
  Node importNode(Node node, optional boolean deep = false);
  [Throws]
  Node adoptNode(Node node);

  [NewObject, Throws]
  Attr createAttribute(DOMString localName);
  [NewObject, Throws]
  Attr createAttributeNS(DOMString? namespace, DOMString qualifiedName);

  [NewObject, Throws]
  Event createEvent(DOMString interface_);

  [NewObject]
  Range createRange();

  // NodeFilter.SHOW_ALL = 0xFFFFFFFF
  [NewObject]
  NodeIterator createNodeIterator(Node root, optional unsigned long whatToShow = 0xFFFFFFFF,
                                  optional NodeFilter? filter = null);
  [NewObject]
  TreeWalker createTreeWalker(Node root, optional unsigned long whatToShow = 0xFFFFFFFF,
                              optional NodeFilter? filter = null);
};

Document implements NonElementParentNode;
Document implements ParentNode;

enum DocumentReadyState { "loading", "interactive", "complete" };

dictionary ElementCreationOptions {
  DOMString is;
};

// https://html.spec.whatwg.org/multipage/#the-document-object
// [OverrideBuiltins]
partial /*sealed*/ interface Document {
  // resource metadata management
  [/*PutForwards=href, */Unforgeable]
  readonly attribute Location? location;
  [SetterThrows] attribute DOMString domain;
  readonly attribute DOMString referrer;
  [Throws]
  attribute DOMString cookie;
  readonly attribute DOMString lastModified;
  readonly attribute DocumentReadyState readyState;

  // DOM tree accessors
     getter object (DOMString name);
           attribute DOMString title;
  //       attribute DOMString dir;
           [SetterThrows]
           attribute HTMLElement? body;
  readonly attribute HTMLHeadElement? head;
  [SameObject]
  readonly attribute HTMLCollection images;
  [SameObject]
  readonly attribute HTMLCollection embeds;
  [SameObject]
  readonly attribute HTMLCollection plugins;
  [SameObject]
  readonly attribute HTMLCollection links;
  [SameObject]
  readonly attribute HTMLCollection forms;
  [SameObject]
  readonly attribute HTMLCollection scripts;
  NodeList getElementsByName(DOMString elementName);
  readonly attribute HTMLScriptElement? currentScript;

  // dynamic markup insertion
  [Throws]
  Document open(optional DOMString type = "text/html", optional DOMString replace = "");
  // WindowProxy open(DOMString url, DOMString name, DOMString features, optional boolean replace = false);
  [Throws]
  void close();
  [Throws]
  void write(DOMString... text);
  [Throws]
  void writeln(DOMString... text);

  // user interaction
  readonly attribute Window?/*Proxy?*/ defaultView;
  readonly attribute Element? activeElement;
  boolean hasFocus();
  // attribute DOMString designMode;
  // boolean execCommand(DOMString commandId, optional boolean showUI = false, optional DOMString value = "");
  // boolean queryCommandEnabled(DOMString commandId);
  // boolean queryCommandIndeterm(DOMString commandId);
  // boolean queryCommandState(DOMString commandId);
  // boolean queryCommandSupported(DOMString commandId);
  // DOMString queryCommandValue(DOMString commandId);

  // special event handler IDL attributes that only apply to Document objects
  [LenientThis] attribute EventHandler onreadystatechange;

  // also has obsolete members
};
Document implements GlobalEventHandlers;
Document implements DocumentAndElementEventHandlers;

// https://html.spec.whatwg.org/multipage/#Document-partial
partial interface Document {
  [TreatNullAs=EmptyString] attribute DOMString fgColor;

  // https://github.com/servo/servo/issues/8715
  // [TreatNullAs=EmptyString] attribute DOMString linkColor;

  // https://github.com/servo/servo/issues/8716
  // [TreatNullAs=EmptyString] attribute DOMString vlinkColor;

  // https://github.com/servo/servo/issues/8717
  // [TreatNullAs=EmptyString] attribute DOMString alinkColor;

  [TreatNullAs=EmptyString] attribute DOMString bgColor;

  [SameObject]
  readonly attribute HTMLCollection anchors;
  [SameObject]
  readonly attribute HTMLCollection applets;

  void clear();
  void captureEvents();
  void releaseEvents();

  // Tracking issue for document.all: https://github.com/servo/servo/issues/7396
  // readonly attribute HTMLAllCollection all;
};

// http://w3c.github.io/touch-events/#idl-def-Document
partial interface Document {
      Touch createTouch(Window/*Proxy*/ view,
                        EventTarget target,
                        long identifier,
                        double pageX,
                        double pageY,
                        double screenX,
                        double screenY);

      TouchList createTouchList(Touch... touches);
};

// https://drafts.csswg.org/cssom-view/#dom-document-elementfrompoint
partial interface Document {
  Element? elementFromPoint(double x, double y);
  sequence<Element> elementsFromPoint(double x, double y);
};

// https://drafts.csswg.org/cssom/#extensions-to-the-document-interface
partial interface Document {
  [SameObject] readonly attribute StyleSheetList styleSheets;
};

// https://fullscreen.spec.whatwg.org/#api
partial interface Document {
  [LenientSetter] readonly attribute boolean fullscreenEnabled;
  [LenientSetter] readonly attribute Element? fullscreenElement;
  [LenientSetter] readonly attribute boolean fullscreen; // historical

  Promise<void> exitFullscreen();

  attribute EventHandler onfullscreenchange;
  attribute EventHandler onfullscreenerror;
};
