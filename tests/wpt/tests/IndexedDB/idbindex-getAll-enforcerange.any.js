// META: title=IndexedDB: IDBIndex getAll() uses [EnforceRange]
// META: global=window,worker
// META: script=resources/support.js
// Spec: "https://w3c.github.io/IndexedDB/#index-interface"
'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('store');
      const index = store.createIndex('index', 'keyPath');
    },
    (t, db) => {
      const tx = db.transaction('store', 'readonly');
      const store = tx.objectStore('store');
      const index = store.index('index');
      [NaN, Infinity, -Infinity, -1, -Number.MAX_SAFE_INTEGER].forEach(
          count => {
            assert_throws_js(TypeError, () => {
              index.getAll(null, count);
            }, `getAll with count ${count} count should throw TypeError`);
          });
      t.done();
    },
    'IDBIndex.getAll() should enforce valid range constraints.');
