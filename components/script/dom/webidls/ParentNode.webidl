/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-parentnode
 */

[NoInterfaceObject]
interface ParentNode {
  [SameObject]
  readonly attribute HTMLCollection children;
  [Pure]
  readonly attribute Element? firstElementChild;
  [Pure]
  readonly attribute Element? lastElementChild;
  [Pure]
  readonly attribute unsigned long childElementCount;

  [Throws, Unscopable]
  void prepend((Node or DOMString)... nodes);
  [Throws, Unscopable]
  void append((Node or DOMString)... nodes);

  [Pure, Throws]
  Element? querySelector(DOMString selectors);
  [NewObject, Throws]
  NodeList querySelectorAll(DOMString selectors);
};
