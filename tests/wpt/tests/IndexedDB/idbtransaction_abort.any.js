// META: title=IDBTransaction - abort
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#transaction-abort

'use strict';

async_test(t => {
  let db;
  let aborted;
  const record = {indexedProperty: 'bar'};

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let txn = e.target.transaction;
    let objStore = db.createObjectStore('store');
    objStore.add(record, 1);
    objStore.add(record, 2);
    let index =
        objStore.createIndex('index', 'indexedProperty', {unique: true});

    assert_true(index instanceof IDBIndex, 'IDBIndex');

    e.target.transaction.onabort = t.step_func(function(e) {
      aborted = true;
      assert_equals(e.type, 'abort', 'event type');
    });

    db.onabort = function(e) {
      assert_true(aborted, 'transaction.abort event has fired');
      t.done();
    };

    e.target.transaction.oncomplete = fail(t, 'got complete, expected abort');
  };
}, 'Abort event should fire during transaction');

indexeddb_test(
    (t, db) => {
      db.createObjectStore('blobs', {keyPath: 'id', autoIncrement: true});
    },
    (t, db) => {
      const txn = db.transaction('blobs', 'readwrite');
      const objectStore = txn.objectStore('blobs');
      const data = new Blob(['test'], {type: 'text/plain'});

      const putRequest = objectStore.put({id: 0, data: data});

      putRequest.onsuccess = t.step_func(() => {
        t.step_timeout(() => {
          assert_throws_dom('InvalidStateError', () => {
            txn.abort();
          }, 'Abort should throw InvalidStateError on an auto-committing transaction.');
        }, 0);
      });

      // Ensure the transaction completes.
      txn.oncomplete = t.step_func(() => {
        t.done();
      });

      // Abort should fail once the transaction has started committing.
      txn.onabort = t.step_func((event) => {
        assert_unreached('Unexpected transaction abort: ' + event.target.error);
      });
      t.add_cleanup(() => {
        if (db) {
          db.close();
        }
      });
    },
    `Abort during auto-committing should throw InvalidStateError.`);

indexeddb_test(
    (t, db) => {
      db.createObjectStore('blobs', {keyPath: 'id', autoIncrement: true});
    },
    (t, db) => {
      const txn = db.transaction('blobs', 'readwrite');
      const objectStore = txn.objectStore('blobs');
      const data = new Blob(['test'], {type: 'text/plain'});

      // Put the object into the store.
      const putRequest = objectStore.put({id: 0, data: data});

      // Handle transaction completion.
      txn.oncomplete = t.step_func(() => {
        assert_throws_dom('InvalidStateError', () => {
          txn.abort();
        }, 'Abort should throw InvalidStateError on a completed transaction.');
        t.done();
      });

      // Handle transaction error.
      txn.onerror = t.step_func((event) => {
        assert_unreached('Unexpected transaction error: ' + event.target.error);
      });

      t.add_cleanup(() => {
        if (db) {
          db.close();
        }
      });
    },
    `Abort on completed transaction should throw InvalidStateError.`);
