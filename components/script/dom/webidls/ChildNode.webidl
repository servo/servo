/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-childnode
 */

[NoInterfaceObject]
interface ChildNode {
  [Throws]
  void before((Node or DOMString)... nodes);
  [Throws]
  void after((Node or DOMString)... nodes);
  [Throws]
  void replaceWith((Node or DOMString)... nodes);
  void remove();
};

[NoInterfaceObject]
interface NonDocumentTypeChildNode {
  [Pure]
  readonly attribute Element? previousElementSibling;
  [Pure]
  readonly attribute Element? nextElementSibling;
};
