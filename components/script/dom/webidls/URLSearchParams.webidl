/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://url.spec.whatwg.org/#interface-urlsearchparams
 */

[Constructor(optional (USVString or URLSearchParams) init/* = ""*/), Exposed=(Window,Worker)]
interface URLSearchParams {
  void append(USVString name, USVString value);
  void delete(USVString name);
  USVString? get(USVString name);
  sequence<USVString> getAll(USVString name);
  boolean has(USVString name);
  void set(USVString name, USVString value);
  // Be careful with implementing iterable interface.
  // Search params might be mutated by URL::SetSearch while iterating (discussed in PR #10351).
  iterable<USVString, USVString>;
  stringifier;
};
