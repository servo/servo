/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://url.spec.whatwg.org/#interface-urlsearchparams
 */

[Constructor(optional (DOMString or URLSearchParams) init)]
interface URLSearchParams {
  void append(DOMString name, DOMString value);
  void delete(DOMString name);
  DOMString? get(DOMString name);
  // sequence<DOMString> getAll(DOMString name);
  boolean has(DOMString name);
  void set(DOMString name, DOMString value);
  //stringifier;
};
