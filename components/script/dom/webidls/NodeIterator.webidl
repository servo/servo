/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * https://dom.spec.whatwg.org/#nodeiterator
 */
// Import from http://hg.mozilla.org/mozilla-central/raw-file/a5a720259d79/dom/webidl/NodeIterator.webidl

interface NodeIterator {
  [SameObject]
  readonly attribute Node root;
  [Pure]
  readonly attribute Node referenceNode;
  [Pure]
  readonly attribute boolean pointerBeforeReferenceNode;
  [Constant]
  readonly attribute unsigned long whatToShow;
  [Constant]
  readonly attribute NodeFilter? filter;

  [Throws]
  Node? nextNode();
  [Throws]
  Node? previousNode();

  [Pure]
  void detach();
};
