/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#range
 * http://domparsing.spec.whatwg.org/#dom-range-createcontextualfragment
 * http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
 */

[Constructor]
interface Range {
  // [Throws]
  // readonly attribute Node startContainer;
  // [Throws]
  // readonly attribute unsigned long startOffset;
  // [Throws]
  // readonly attribute Node endContainer;
  // [Throws]
  // readonly attribute unsigned long endOffset;
  // readonly attribute boolean collapsed;
  // [Throws]
  // readonly attribute Node commonAncestorContainer;

  // [Throws]
  // void setStart(Node refNode, unsigned long offset);
  // [Throws]
  // void setEnd(Node refNode, unsigned long offset);
  // [Throws]
  // void setStartBefore(Node refNode);
  // [Throws]
  // void setStartAfter(Node refNode);
  // [Throws]
  // void setEndBefore(Node refNode);
  // [Throws]
  // void setEndAfter(Node refNode);
  // void collapse(optional boolean toStart = false);
  // [Throws]
  // void selectNode(Node refNode);
  // [Throws]
  // void selectNodeContents(Node refNode);

  // const unsigned short START_TO_START = 0;
  // const unsigned short START_TO_END = 1;
  // const unsigned short END_TO_END = 2;
  // const unsigned short END_TO_START = 3;
  // [Throws]
  // short compareBoundaryPoints(unsigned short how, Range sourceRange);
  // [Throws]
  // void deleteContents();
  // [Throws]
  // DocumentFragment extractContents();
  // [Throws]
  // DocumentFragment cloneContents();
  // [Throws]
  // void insertNode(Node node);
  // [Throws]
  // void surroundContents(Node newParent);

  // Range cloneRange();
  void detach();

  // [Throws]
  // boolean isPointInRange(Node node, unsigned long offset);
  // [Throws]
  // short comparePoint(Node node, unsigned long offset);

  // [Throws]
  // boolean intersectsNode(Node node);

  // stringifier;
};

// http://domparsing.spec.whatwg.org/#dom-range-createcontextualfragment
partial interface Range {
  // [Throws]
  // DocumentFragment createContextualFragment(DOMString fragment);
};// 

////  http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
partial interface Range {
  // DOMRectList? getClientRects();
  // DOMRect getBoundingClientRect();
};
