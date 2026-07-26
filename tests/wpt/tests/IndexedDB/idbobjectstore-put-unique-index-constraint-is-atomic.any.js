// META: title=IndexedDB: a put() that fails a unique index constraint must be atomic
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#object-store-storage-operation

'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('store', {keyPath: 'id'});
      store.createIndex('username', 'username', {unique: true});
      store.add({id: 1, username: 'foo'});
      store.add({id: 2, username: 'bar'});
    },
    (t, db) => {
      // Overwriting id:1 with username 'bar' collides with id:2's unique index
      // entry, so the put() must fail with a ConstraintError and leave the
      // store exactly as it was.
      const tx = db.transaction('store', 'readwrite');
      const store = tx.objectStore('store');
      const request = store.put({id: 1, username: 'bar'});

      request.onsuccess =
          t.unreached_func('put() should fail with a ConstraintError');
      request.onerror = t.step_func(e => {
        assert_equals(request.error.name, 'ConstraintError');
        // Handle the error so the transaction commits instead of aborting;
        // this is what exposes the non-atomic delete-then-add bug.
        e.preventDefault();
      });

      tx.onabort =
          t.unreached_func('transaction should commit, not abort');
      tx.oncomplete = t.step_func(() => {
        const readTx = db.transaction('store', 'readonly');
        const readStore = readTx.objectStore('store');

        const getRequest = readStore.get(1);
        getRequest.onsuccess = t.step_func(() => {
          assert_not_equals(
              getRequest.result, undefined,
              'the record that would have been overwritten must still exist');
          assert_equals(
              getRequest.result.username, 'foo',
              'the pre-existing record must be unchanged');
        });

        const countRequest = readStore.count();
        countRequest.onsuccess = t.step_func(() => {
          assert_equals(
              countRequest.result, 2,
              'the store must still contain both original records');
        });

        readTx.oncomplete = t.step_func_done();
      });
    },
    'A put() that fails a unique index constraint must not delete the record ' +
        'it would have overwritten');
