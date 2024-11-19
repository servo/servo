/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-childnode
 */

interface mixin ChildNode {
  [Throws, CEReactions, Unscopable]
  undefined before((Node or DOMString)... nodes);
  [Throws, CEReactions, Unscopable]
  undefined after((Node or DOMString)... nodes);
  [Throws, CEReactions, Unscopable]
  undefined replaceWith((Node or DOMString)... nodes);
  [CEReactions, Unscopable]
  undefined remove();
};

interface mixin NonDocumentTypeChildNode {
  [Pure]
  readonly attribute Element? previousElementSibling;
  [Pure]
  readonly attribute Element? nextElementSibling;
};
