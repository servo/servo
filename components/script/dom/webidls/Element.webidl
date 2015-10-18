/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#element and
 * https://domparsing.spec.whatwg.org/ and
 * http://dev.w3.org/csswg/cssom-view/ and
 * http://www.w3.org/TR/selectors-api/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface Element : Node {
  [Constant]
  readonly attribute DOMString? namespaceURI;
  [Constant]
  readonly attribute DOMString? prefix;
  [Constant]
  readonly attribute DOMString localName;
  // Not [Constant] because it depends on which document we're in
  [Pure]
  readonly attribute DOMString tagName;

  [Pure]
           attribute DOMString id;
  [Pure]
           attribute DOMString className;
  [SameObject]
  readonly attribute DOMTokenList classList;

  [SameObject]
  readonly attribute NamedNodeMap attributes;
  [Pure]
  DOMString? getAttribute(DOMString name);
  [Pure]
  DOMString? getAttributeNS(DOMString? namespace, DOMString localName);
  [Pure]
  Attr? getAttributeNode(DOMString name);
  [Pure]
  Attr? getAttributeNodeNS(DOMString? namespace, DOMString localName);
  [Throws]
  void setAttribute(DOMString name, DOMString value);
  [Throws]
  void setAttributeNS(DOMString? namespace, DOMString name, DOMString value);
  void removeAttribute(DOMString name);
  void removeAttributeNS(DOMString? namespace, DOMString localName);
  boolean hasAttribute(DOMString name);
  boolean hasAttributeNS(DOMString? namespace, DOMString localName);

  [Pure, Throws]
  Element? closest(DOMString selectors);

  [Pure, Throws]
  boolean matches(DOMString selectors);

  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-element-interface
partial interface Element {
  DOMRectList getClientRects();
  DOMRect getBoundingClientRect();

  readonly attribute long clientTop;
  readonly attribute long clientLeft;
  readonly attribute long clientWidth;
  readonly attribute long clientHeight;
};

// https://domparsing.spec.whatwg.org/#extensions-to-the-element-interface
partial interface Element {
  [Throws,TreatNullAs=EmptyString]
  attribute DOMString innerHTML;
  [Throws,TreatNullAs=EmptyString]
  attribute DOMString outerHTML;
};

Element implements ChildNode;
Element implements NonDocumentTypeChildNode;
Element implements ParentNode;
