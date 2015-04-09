/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is:
 * http://dom.spec.whatwg.org/#interface-document
 * http://www.whatwg.org/specs/web-apps/current-work/#the-document-object
 */

/* http://dom.spec.whatwg.org/#interface-document */
[Constructor]
interface Document : Node {
  readonly attribute DOMImplementation implementation;
  readonly attribute DOMString URL;
  readonly attribute Element? activeElement;
  readonly attribute DOMString documentURI;
  readonly attribute DOMString compatMode;
  readonly attribute DOMString characterSet;
  readonly attribute DOMString inputEncoding;
  readonly attribute DOMString contentType;

  readonly attribute DocumentType? doctype;
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);
  Element? getElementById(DOMString elementId);

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
  // [NewObject]
  // NodeIterator createNodeIterator(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
  [NewObject]
  TreeWalker createTreeWalker(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
};
Document implements ParentNode;

enum DocumentReadyState { "loading", "interactive", "complete" };

/* http://www.whatwg.org/specs/web-apps/current-work/#the-document-object */
partial interface Document {
  // resource metadata management
  readonly attribute DocumentReadyState readyState;
  readonly attribute DOMString lastModified;
  readonly attribute Location location;
  [Throws]
  attribute DOMString cookie;

  // DOM tree accessors
           [SetterThrows]
           attribute DOMString title;
           [SetterThrows]
           attribute HTMLElement? body;
  readonly attribute HTMLHeadElement? head;
  readonly attribute HTMLCollection images;
  readonly attribute HTMLCollection embeds;
  readonly attribute HTMLCollection plugins;
  readonly attribute HTMLCollection links;
  readonly attribute HTMLCollection forms;
  readonly attribute HTMLCollection scripts;
  readonly attribute HTMLCollection anchors;
  readonly attribute HTMLCollection applets;
  NodeList getElementsByName(DOMString elementName);
  readonly attribute HTMLScriptElement? currentScript;

  // special event handler IDL attributes that only apply to Document objects
  [LenientThis] attribute EventHandler onreadystatechange;

  // user interaction
  readonly attribute Window defaultView;
};
Document implements GlobalEventHandlers;
