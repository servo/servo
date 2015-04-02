/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

interface NamedNodeMap {
  readonly attribute unsigned long length;
  getter Attr? item(unsigned long index);
  getter Attr? getNamedItem(DOMString name);
  Attr? getNamedItemNS(DOMString? namespace, DOMString localName);
  //[Throws]
  //Attr? setNamedItem(Attr attr);
  //[Throws]
  //Attr? setNamedItemNS(Attr attr);
  //[Throws]
  //Attr removeNamedItem(DOMString name);
  //[Throws]
  //Attr removeNamedItemNS(DOMString? namespace, DOMString name);
};
