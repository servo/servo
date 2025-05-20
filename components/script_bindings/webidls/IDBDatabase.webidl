/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbdatabase
 *
 */

// https://w3c.github.io/IndexedDB/#idbdatabase
[Pref="dom_indexeddb_enabled", Exposed=(Window,Worker)]
interface IDBDatabase : EventTarget {
  readonly attribute DOMString name;
  readonly attribute unsigned long long version;
  readonly attribute DOMStringList objectStoreNames;

  [Throws, NewObject] IDBTransaction transaction((DOMString or sequence<DOMString>) storeNames,
                                         optional IDBTransactionMode mode = "readonly",
                                         optional IDBTransactionOptions options = {});
  undefined close();

  [Throws, NewObject] IDBObjectStore createObjectStore(
    DOMString name,
    optional IDBObjectStoreParameters options = {}
  );
  [Throws] undefined deleteObjectStore(DOMString name);

  // Event handlers:
  attribute EventHandler onabort;
  attribute EventHandler onclose;
  attribute EventHandler onerror;
  attribute EventHandler onversionchange;
};

enum IDBTransactionDurability { "default", "strict", "relaxed" };

dictionary IDBTransactionOptions {
  IDBTransactionDurability durability = "default";
};

dictionary IDBObjectStoreParameters {
  (DOMString or sequence<DOMString>)? keyPath = null;
  boolean autoIncrement = false;
};
