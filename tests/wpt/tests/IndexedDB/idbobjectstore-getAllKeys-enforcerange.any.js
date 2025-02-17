// META: global=window,worker
// META: title=IndexedDB: IDBIObjectStore getAllKeys() uses [EnforceRange]
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#object-store-interface

'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('store');
    },
    (t, db) => {
      const tx = db.transaction('store', 'readonly');
      const store = tx.objectStore('store');
      [NaN, Infinity, -Infinity, -1, -Number.MAX_SAFE_INTEGER].forEach(
          count => {
            assert_throws_js(TypeError, () => {
              store.getAllKeys(null, count);
            }, `getAllKeys with count ${count} count should throw TypeError`);
          });
      t.done();
    },
    `IDBObjectStore.getAllKeys() uses [EnforceRange]`);
