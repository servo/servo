/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://url.spec.whatwg.org/#interface-urlsearchparams
 */

[Exposed=(Window,Worker)]
interface URLSearchParams {
  [Throws] constructor(optional (sequence<sequence<USVString>> or record<USVString, USVString> or USVString) init = "");
  readonly attribute unsigned long size;
  undefined append(USVString name, USVString value);
  undefined delete(USVString name, optional USVString value);
  USVString? get(USVString name);
  sequence<USVString> getAll(USVString name);
  boolean has(USVString name, optional USVString value);
  undefined set(USVString name, USVString value);

  undefined sort();

  // Be careful with implementing iterable interface.
  // Search params might be mutated by URL::SetSearch while iterating (discussed in PR #10351).
  iterable<USVString, USVString>;
  stringifier;
};
