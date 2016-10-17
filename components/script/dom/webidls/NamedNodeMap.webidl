/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#interface-namednodemap

[LegacyUnenumerableNamedProperties]
interface NamedNodeMap {
  [Pure]
  readonly attribute unsigned long length;
  [Pure]
  getter Attr? item(unsigned long index);
  [Pure]
  getter Attr? getNamedItem(DOMString qualifiedName);
  [Pure]
  Attr? getNamedItemNS(DOMString? namespace, DOMString localName);
  [Throws]
  Attr? setNamedItem(Attr attr);
  [Throws]
  Attr? setNamedItemNS(Attr attr);
  [Throws]
  Attr removeNamedItem(DOMString qualifiedName);
  [Throws]
  Attr removeNamedItemNS(DOMString? namespace, DOMString localName);
};
