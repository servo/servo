/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-attr
 *
 */

interface Attr {
  [Constant]
  readonly attribute DOMString? namespaceURI;
  [Constant]
  readonly attribute DOMString? prefix;
  [Constant]
  readonly attribute DOMString localName;
  [Constant]
  readonly attribute DOMString name;
  [Pure]
           attribute DOMString value;
  [Pure]
           attribute DOMString textContent; // alias of .value
  [Pure]
           attribute DOMString nodeValue; // alias of .value

  [Pure]
  readonly attribute Element? ownerElement;

  [Constant]
  readonly attribute boolean specified; // useless; always returns true
};
