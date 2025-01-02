// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

idl_test(
  ['IndexedDB'],
  ['html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      IDBCursor: [],
      IDBCursorWithValue: [],
      IDBDatabase: [],
      IDBFactory: [self.indexedDB],
      IDBIndex: [],
      IDBKeyRange: [IDBKeyRange.only(0)],
      IDBObjectStore: [],
      IDBOpenDBRequest: [],
      IDBRequest: [],
      IDBTransaction: [],
      IDBVersionChangeEvent: ['new IDBVersionChangeEvent("type")'],
      DOMStringList: [],
    });
  }
);
