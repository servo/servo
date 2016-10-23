/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#domtokenlist
interface DOMTokenList {
  [Pure]
  readonly attribute unsigned long length;
  [Pure]
  getter DOMString? item(unsigned long index);

  [Pure]
  boolean contains(DOMString token);
  [Throws]
  void add(DOMString... tokens);
  [Throws]
  void remove(DOMString... tokens);
  [Throws]
  boolean toggle(DOMString token, optional boolean force);
  [Throws]
  void replace(DOMString token, DOMString newToken);

  [Pure]
           attribute DOMString value;

  stringifier;
  iterable<DOMString?>;
};
