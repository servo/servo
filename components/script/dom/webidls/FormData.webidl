/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://xhr.spec.whatwg.org
 */

typedef (File or DOMString) FormDataEntryValue;

[Constructor(optional HTMLFormElement form)]
interface FormData {
  void append(DOMString name, Blob value, optional DOMString filename);
  void append(DOMString name, DOMString value);
  void delete(DOMString name);
  FormDataEntryValue? get(DOMString name);
  // sequence<FormDataEntryValue> getAll(DOMString name);
  boolean has(DOMString name);
  void set(DOMString name, Blob value, optional DOMString filename);
  void set(DOMString name, DOMString value);
};
