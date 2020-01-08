/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#range
 * https://w3c.github.io/DOM-Parsing/#dom-range-createcontextualfragment
 * http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
 */

[Exposed=Window]
interface Range : AbstractRange {
  [Throws] constructor();
  [Pure]
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
  [Pure, Throws]
  short compareBoundaryPoints(unsigned short how, Range sourceRange);
  [CEReactions, Throws]
  void deleteContents();
  [CEReactions, NewObject, Throws]
  DocumentFragment extractContents();
  [CEReactions, NewObject, Throws]
  DocumentFragment cloneContents();
  [CEReactions, Throws]
  void insertNode(Node node);
  [CEReactions, Throws]
  void surroundContents(Node newParent);

  [NewObject]
  Range cloneRange();
  [Pure]
  void detach();

  [Pure, Throws]
  boolean isPointInRange(Node node, unsigned long offset);
  [Pure, Throws]
  short comparePoint(Node node, unsigned long offset);

  [Pure]
  boolean intersectsNode(Node node);

  stringifier;
};

// https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#extensions-to-the-range-interface
partial interface Range {
  [CEReactions, NewObject, Throws]
  DocumentFragment createContextualFragment(DOMString fragment);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-range-interface
partial interface Range {
  // sequence<DOMRect> getClientRects();
  // [NewObject]
  // DOMRect getBoundingClientRect();
};
