/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbcursorwithvalue
 *
 */

// https://w3c.github.io/IndexedDB/#idbcursorwithvalue
[Pref="dom_indexeddb_enabled", Exposed=(Window,Worker)]
interface IDBCursorWithValue : IDBCursor {
  readonly attribute any value;
};
