// META: title=IndexedDB: Verify [SameObject] behavior of IDBTransaction's db attribute
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbtransaction-db

'use strict';

indexeddb_test(
    (t, db, tx) => {
      const store = db.createObjectStore('store');
      assert_equals(
          tx.db, tx.db, 'Attribute should yield the same object each time');
    },
    (t, db) => {
      const tx = db.transaction('store', 'readonly');
      assert_equals(
          tx.db, tx.db, 'Attribute should yield the same object each time');
      t.done();
    },
    'IDBTransaction.db [SameObject]');
