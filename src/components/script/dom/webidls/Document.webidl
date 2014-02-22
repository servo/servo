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
  readonly attribute DOMString documentURI;
  readonly attribute DOMString compatMode;
  readonly attribute DOMString characterSet;
  readonly attribute DOMString contentType;

  readonly attribute DocumentType? doctype;
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);
  Element? getElementById(DOMString elementId);

  [Creator, Throws]
  Element createElement(DOMString localName);
  [Creator]
  DocumentFragment createDocumentFragment();
  [Creator]
  Text createTextNode(DOMString data);
  [Creator]
  Comment createComment(DOMString data);
  [Creator, Throws]
  ProcessingInstruction createProcessingInstruction(DOMString target, DOMString data);

  [Creator, Throws]
  Event createEvent(DOMString interface_);
};

/* http://www.whatwg.org/specs/web-apps/current-work/#the-document-object */
partial interface Document {
           [SetterThrows]
           attribute DOMString title;
           attribute HTMLElement? body;
  readonly attribute HTMLHeadElement? head;
  /*NodeList*/ HTMLCollection getElementsByName(DOMString elementName);
};
