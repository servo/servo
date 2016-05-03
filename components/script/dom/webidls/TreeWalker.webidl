/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-treewalker
 */

interface TreeWalker {
  [SameObject]
  readonly attribute Node root;
  [Constant]
  readonly attribute unsigned long whatToShow;
  [Constant]
  readonly attribute NodeFilter? filter;
  [Pure]
           attribute Node currentNode;

  [Throws]
  Node? parentNode();
  [Throws]
  Node? firstChild();
  [Throws]
  Node? lastChild();
  [Throws]
  Node? previousSibling();
  [Throws]
  Node? nextSibling();
  [Throws]
  Node? previousNode();
  [Throws]
  Node? nextNode();
};
