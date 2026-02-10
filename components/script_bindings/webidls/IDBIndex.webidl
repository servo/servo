/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbindex
 *
 */

[Pref="dom_indexeddb_enabled", Exposed=(Window,Worker)]
interface IDBIndex {
  // attribute DOMString name;
  [SameObject] readonly attribute IDBObjectStore objectStore;
  readonly attribute any keyPath;
  readonly attribute boolean multiEntry;
  readonly attribute boolean unique;

  // [NewObject] IDBRequest get(any query);
  // [NewObject] IDBRequest getKey(any query);
  // [NewObject] IDBRequest getAll(optional any queryOrOptions,
  //                               optional [EnforceRange] unsigned long count);
  // [NewObject] IDBRequest getAllKeys(optional any queryOrOptions,
  //                                   optional [EnforceRange] unsigned long count);
  // [NewObject] IDBRequest getAllRecords(optional IDBGetAllOptions options = {});
  // [NewObject] IDBRequest count(optional any query);

  // [NewObject] IDBRequest openCursor(optional any query,
  //                                   optional IDBCursorDirection direction = "next");
  // [NewObject] IDBRequest openKeyCursor(optional any query,
  //                                      optional IDBCursorDirection direction = "next");
};
