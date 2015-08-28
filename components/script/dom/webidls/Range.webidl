/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#range
 * https://domparsing.spec.whatwg.org/#dom-range-createcontextualfragment
 * http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
 */

[Constructor /*, Exposed=Window */]
interface Range {
  readonly attribute Node startContainer;
  readonly attribute unsigned long startOffset;
  readonly attribute Node endContainer;
  readonly attribute unsigned long endOffset;
  readonly attribute boolean collapsed;
  readonly attribute Node commonAncestorContainer;

  [Throws]
  void setStart(Node refNode, unsigned long offset);
  [Throws]
  void setEnd(Node refNode, unsigned long offset);
  [Throws]
  void setStartBefore(Node refNode);
  [Throws]
  void setStartAfter(Node refNode);
  [Throws]
  void setEndBefore(Node refNode);
  [Throws]
  void setEndAfter(Node refNode);
  void collapse(optional boolean toStart = false);
  [Throws]
  void selectNode(Node refNode);
  [Throws]
  void selectNodeContents(Node refNode);

  const unsigned short START_TO_START = 0;
  const unsigned short START_TO_END = 1;
  const unsigned short END_TO_END = 2;
  const unsigned short END_TO_START = 3;
  [Throws]
  short compareBoundaryPoints(unsigned short how, Range sourceRange);
  // [Throws]
  // void deleteContents();
  [NewObject, Throws]
  DocumentFragment extractContents();
  [NewObject, Throws]
  DocumentFragment cloneContents();
  [Throws]
  void insertNode(Node node);
  [Throws]
  void surroundContents(Node newParent);

  [NewObject]
  Range cloneRange();
  void detach();

  [Throws]
  boolean isPointInRange(Node node, unsigned long offset);
  [Throws]
  short comparePoint(Node node, unsigned long offset);

  boolean intersectsNode(Node node);

  // stringifier;
};

// https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#extensions-to-the-range-interface
partial interface Range {
  // [NewObject, Throws]
  // DocumentFragment createContextualFragment(DOMString fragment);
};//

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-range-interface
partial interface Range {
  // DOMRectList? getClientRects();
  // DOMRect getBoundingClientRect();
};
