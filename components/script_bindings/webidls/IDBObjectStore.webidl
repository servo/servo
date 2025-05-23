/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbobjectstore
 *
 */

// https://w3c.github.io/IndexedDB/#idbobjectstore
[Pref="dom_indexeddb_enabled", Exposed=(Window,Worker)]
interface IDBObjectStore {
  attribute DOMString name;
  // readonly attribute any keyPath;
  // readonly attribute DOMStringList indexNames;
  // [SameObject] readonly attribute IDBTransaction transaction;
  readonly attribute boolean autoIncrement;

  [NewObject, Throws] IDBRequest put(any value, optional any key);
  [NewObject, Throws] IDBRequest add(any value, optional any key);
  [NewObject, Throws] IDBRequest delete(any query);
  [NewObject, Throws] IDBRequest clear();
  [NewObject, Throws] IDBRequest get(any query);
  // [NewObject] IDBRequest getKey(any query);
  // [NewObject] IDBRequest getAll(optional any query,
                                // optional [EnforceRange] unsigned long count);
  // [NewObject] IDBRequest getAllKeys(optional any query,
                                    // optional [EnforceRange] unsigned long count);
  [NewObject, Throws] IDBRequest count(optional any query);

  // [NewObject] IDBRequest openCursor(optional any query,
  //                                   optional IDBCursorDirection direction = "next");
  // [NewObject] IDBRequest openKeyCursor(optional any query,
  //                                      optional IDBCursorDirection direction = "next");

  // IDBIndex index(DOMString name);

  // [NewObject] IDBIndex createIndex(DOMString name,
  //                                  (DOMString or sequence<DOMString>) keyPath,
  //                                  optional IDBIndexParameters options = {});
  // void deleteIndex(DOMString name);
};

// https://w3c.github.io/IndexedDB/#dictdef-idbindexparameters
dictionary IDBIndexParameters {
  boolean unique = false;
  boolean multiEntry = false;
};
