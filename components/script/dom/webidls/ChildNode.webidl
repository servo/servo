/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-childnode
 */

[NoInterfaceObject]
interface ChildNode {
  [Throws, Unscopable]
  void before((Node or DOMString)... nodes);
  [Throws, Unscopable]
  void after((Node or DOMString)... nodes);
  [Throws, Unscopable]
  void replaceWith((Node or DOMString)... nodes);
  [Unscopable]
  void remove();
};

[NoInterfaceObject]
interface NonDocumentTypeChildNode {
  [Pure]
  readonly attribute Element? previousElementSibling;
  [Pure]
  readonly attribute Element? nextElementSibling;
};
