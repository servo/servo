/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#interface-attr
 *
 */

interface Attr {
  readonly attribute DOMString localName;
           attribute DOMString value;

  readonly attribute DOMString name;
  readonly attribute DOMString? namespaceURI;
  readonly attribute DOMString? prefix;
};
