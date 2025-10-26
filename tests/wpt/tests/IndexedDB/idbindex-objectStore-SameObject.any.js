// META: title=IndexedDB: objectStore SameObject
// META: global=window,worker
// META: script=resources/support.js
// Spec: "https://w3c.github.io/IndexedDB/#dom-idbindex-objectstore"
'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('store');
      const index = store.createIndex('index', 'keyPath');
      assert_equals(
          index.objectStore, index.objectStore,
          'Attribute should yield the same object each time');
    },
    (t, db) => {
      const tx = db.transaction('store', 'readonly');
      const store = tx.objectStore('store');
      const index = store.index('index');
      assert_equals(
          index.objectStore, index.objectStore,
          'Attribute should yield the same object each time');
      t.done();
    },
    'IDBIndex.objectStore should return same object each time.');
