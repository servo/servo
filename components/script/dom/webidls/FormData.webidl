/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://xhr.spec.whatwg.org/#interface-formdata
 */

typedef (File or USVString) FormDataEntryValue;

[Exposed=(Window,Worker)]
interface FormData {
  [Throws] constructor(optional HTMLFormElement form);
  undefined append(USVString name, USVString value);
  undefined append(USVString name, Blob value, optional USVString filename);
  undefined delete(USVString name);
  FormDataEntryValue? get(USVString name);
  sequence<FormDataEntryValue> getAll(USVString name);
  boolean has(USVString name);
  undefined set(USVString name, USVString value);
  undefined set(USVString name, Blob value, optional USVString filename);
  iterable<USVString, FormDataEntryValue>;
};
