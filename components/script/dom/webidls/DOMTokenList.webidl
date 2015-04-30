/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#domtokenlist
interface DOMTokenList {
  readonly attribute unsigned long length;
  getter DOMString? item(unsigned long index);

  [Throws]
  boolean contains(DOMString token);
  [Throws]
  void add(DOMString... tokens);
  [Throws]
  void remove(DOMString... tokens);
  [Throws]
  boolean toggle(DOMString token, optional boolean force);

  stringifier;
};
