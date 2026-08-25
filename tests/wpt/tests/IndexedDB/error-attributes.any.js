// META: title=IndexedDB
// META: global=window,worker
// META: script=resources/support.js

'use strict';

indexeddb_test(
    function(t, db) {
      db.createObjectStore('store');
    },
    function(t, db) {
      let tx = db.transaction('store', 'readwrite');
      let store = tx.objectStore('store');
      let r1 = store.add('value', 'key');
      r1.onerror = t.unreached_func('first add should succeed');

      let r2 = store.add('value', 'key');
      r2.onsuccess = t.unreached_func('second add should fail');

      r2.onerror = t.step_func(function() {
        assert_true(r2.error instanceof DOMException);
        assert_equals(r2.error.name, 'ConstraintError');
      });

      tx.oncomplete = t.unreached_func('transaction should not complete');
      tx.onabort = t.step_func(function() {
        assert_true(tx.error instanceof DOMException);
        assert_equals(tx.error.name, 'ConstraintError');
        t.done();
      });
    },
    'IDBRequest and IDBTransaction error properties should be DOMExceptions');
