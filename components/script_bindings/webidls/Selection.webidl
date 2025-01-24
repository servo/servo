/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/selection-api/#selection-interface
[Exposed=Window]
interface Selection {
readonly attribute Node? anchorNode;
  readonly attribute unsigned long anchorOffset;
  readonly attribute Node? focusNode;
  readonly attribute unsigned long focusOffset;
  readonly attribute boolean isCollapsed;
  readonly attribute unsigned long rangeCount;
  readonly attribute DOMString type;
  [Throws] Range getRangeAt(unsigned long index);
  undefined addRange(Range range);
  [Throws] undefined removeRange(Range range);
  undefined removeAllRanges();
  undefined empty();
  [Throws] undefined collapse(Node? node, optional unsigned long offset = 0);
  [Throws] undefined setPosition(Node? node, optional unsigned long offset = 0);
  [Throws] undefined collapseToStart();
  [Throws] undefined collapseToEnd();
  [Throws] undefined extend(Node node, optional unsigned long offset = 0);
  [Throws]
  undefined setBaseAndExtent(Node anchorNode, unsigned long anchorOffset, Node focusNode, unsigned long focusOffset);
  [Throws] undefined selectAllChildren(Node node);
  [CEReactions, Throws]
  undefined deleteFromDocument();
  boolean containsNode(Node node, optional boolean allowPartialContainment = false);
  stringifier DOMString ();
};
