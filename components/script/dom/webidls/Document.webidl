/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-document
 * https://www.whatwg.org/specs/web-apps/current-work/#the-document-object
 */

// https://dom.spec.whatwg.org/#interface-document
[Constructor]
interface Document : Node {
  [SameObject]
  readonly attribute DOMImplementation implementation;
  readonly attribute DOMString URL;
  readonly attribute Element? activeElement;
  readonly attribute DOMString documentURI;
  readonly attribute DOMString compatMode;
  readonly attribute DOMString characterSet;
  readonly attribute DOMString inputEncoding; // legacy alias of .characterSet
  readonly attribute DOMString contentType;

  readonly attribute DocumentType? doctype;
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);

  [NewObject, Throws]
  Element createElement(DOMString localName);
  [NewObject, Throws]
  Element createElementNS(DOMString? namespace, DOMString qualifiedName);
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
  Attr createAttributeNS(DOMString? namespace, DOMString localName);

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

// https://www.whatwg.org/specs/web-apps/current-work/#the-document-object
// [OverrideBuiltins]
partial /*sealed*/ interface Document {
  // resource metadata management
  // [PutForwards=href, Unforgeable]
  readonly attribute Location/*?*/ location;
  // attribute DOMString domain;
  // readonly attribute DOMString referrer;
  [Throws]
  attribute DOMString cookie;
  readonly attribute DOMString lastModified;
  readonly attribute DocumentReadyState readyState;

  // DOM tree accessors
     getter object (DOMString name);
           attribute DOMString title;
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
  // NodeList getItems(optional DOMString typeNames = ""); // microdata
  // [SameObject]
  // readonly attribute DOMElementMap cssElementMap;
  readonly attribute HTMLScriptElement? currentScript;

  // dynamic markup insertion
  // Document open(optional DOMString type = "text/html", optional DOMString replace = "");
  // WindowProxy open(DOMString url, DOMString name, DOMString features, optional boolean replace = false);
  // void close();
  // void write(DOMString... text);
  // void writeln(DOMString... text);

  // user interaction
  readonly attribute Window/*Proxy?*/ defaultView;
  // readonly attribute Element? activeElement;
  boolean hasFocus();
  // attribute DOMString designMode;
  // boolean execCommand(DOMString commandId, optional boolean showUI = false, optional DOMString value = "");
  // boolean queryCommandEnabled(DOMString commandId);
  // boolean queryCommandIndeterm(DOMString commandId);
  // boolean queryCommandState(DOMString commandId);
  // boolean queryCommandSupported(DOMString commandId);
  // DOMString queryCommandValue(DOMString commandId);
  // readonly attribute HTMLCollection commands;

  // special event handler IDL attributes that only apply to Document objects
  [LenientThis] attribute EventHandler onreadystatechange;

  // also has obsolete members
};
Document implements GlobalEventHandlers;

// https://html.spec.whatwg.org/#Document-partial
partial interface Document {
  // [TreatNullAs=EmptyString] attribute DOMString fgColor;
  // [TreatNullAs=EmptyString] attribute DOMString linkColor;
  // [TreatNullAs=EmptyString] attribute DOMString vlinkColor;
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
