// META: global=window,worker
// META: title=IndexedDB: IDBObjectStore index() when transaction is finished
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbobjectstore-index

'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('store');
      store.createIndex('index', 'key_path');
    },
    (t, db) => {
      const tx = db.transaction('store', 'readonly');
      const store = tx.objectStore('store');
      tx.abort();
      assert_throws_dom(
          'InvalidStateError', () => store.index('index'),
          'index() should throw if transaction is finished');
      t.done();
    },
    'IDBObjectStore index() behavior when transaction is finished');
