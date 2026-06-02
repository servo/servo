/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-parentnode
 */

interface mixin ParentNode {
  [SameObject]
  readonly attribute HTMLCollection children;
  [Pure]
  readonly attribute Element? firstElementChild;
  [Pure]
  readonly attribute Element? lastElementChild;
  [Pure]
  readonly attribute unsigned long childElementCount;

  [CEReactions, Throws, Unscopable]
  undefined prepend((Node or DOMString)... nodes);
  [CEReactions, Throws, Unscopable]
  undefined append((Node or DOMString)... nodes);
  [CEReactions, Throws, Unscopable]
  undefined replaceChildren((Node or DOMString)... nodes);

  [Pure, Throws]
  Element? querySelector(DOMString selectors);
  [NewObject, Throws]
  NodeList querySelectorAll(DOMString selectors);
};
