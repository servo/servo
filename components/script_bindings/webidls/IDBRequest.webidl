/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbrequest
 *
 */

// https://w3c.github.io/IndexedDB/#idbrequest
[Pref="dom_indexeddb_enabled", Exposed=(Window,Worker)]
interface IDBRequest : EventTarget {
  readonly attribute any result;
  readonly attribute DOMException? error;
  // readonly attribute (IDBObjectStore or IDBIndex or IDBCursor)? source;
  readonly attribute IDBObjectStore? source;
  readonly attribute IDBTransaction? transaction;
  readonly attribute IDBRequestReadyState readyState;

  // Event handlers:
  attribute EventHandler onsuccess;
  attribute EventHandler onerror;
};

// https://w3c.github.io/IndexedDB/#enumdef-idbrequestreadystate
enum IDBRequestReadyState {
  "pending",
  "done"
};
