/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/IndexedDB/#idbversionchangeevent
 *
 */

// FIXME:(arihant2math) Expose to Worker too

// https://w3c.github.io/IndexedDB/#idbversionchangeevent
[Pref="dom.indexeddb.enabled", Exposed=(Window)]
interface IDBVersionChangeEvent : Event {
  constructor(DOMString type, optional IDBVersionChangeEventInit eventInitDict = {});

  readonly attribute unsigned long long oldVersion;
  readonly attribute unsigned long long? newVersion;
};

// https://w3c.github.io/IndexedDB/#idbversionchangeevent
dictionary IDBVersionChangeEventInit : EventInit {
  unsigned long long oldVersion = 0;
  unsigned long long? newVersion = null;
};
