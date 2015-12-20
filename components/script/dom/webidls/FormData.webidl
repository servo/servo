/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://xhr.spec.whatwg.org/#interface-formdata
 */

typedef (Blob or USVString) FormDataEntryValue;

[Constructor(optional HTMLFormElement form),
 /*Exposed=(Window,Worker)*/]
interface FormData {
  void append(USVString name, USVString value);
  void append(USVString name, Blob value, optional USVString filename);
  void delete(USVString name);
  FormDataEntryValue? get(USVString name);
  sequence<FormDataEntryValue> getAll(USVString name);
  boolean has(USVString name);
  void set(USVString name, FormDataEntryValue value);
  // iterable<USVString, FormDataEntryValue>;
};
