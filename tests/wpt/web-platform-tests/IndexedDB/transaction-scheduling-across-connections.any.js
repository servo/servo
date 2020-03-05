// META: script=support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store');
  },

  (t, db1) => {
    // Open a second connection to the same database.
    const open_request = indexedDB.open(db1.name);
    open_request.onerror = t.unreached_func('open() should succeed');
    open_request.onupgradeneeded =
      t.unreached_func('second connection should not upgrade');
    open_request.onsuccess = t.step_func(() => {
      const db2 = open_request.result;
      t.add_cleanup(() => { db2.close(); });

      const transaction1 = db1.transaction('store', 'readwrite');
      transaction1.onabort = t.unreached_func('transaction1 should complete');

      const transaction2 = db2.transaction('store', 'readwrite');
      transaction2.onabort = t.unreached_func('transaction2 should complete');

      let transaction1PutSuccess = false;
      let transaction1Complete = false;
      let transaction2PutSuccess = false;

      // Keep transaction1 alive for a while and ensure transaction2
      // doesn't start.

      let count = 0;
      (function doTransaction1Put() {
        const request = transaction1.objectStore('store').put(1, count++);
        request.onerror = t.unreached_func('request should succeed');
        request.onsuccess = t.step_func(evt => {
          transaction1PutSuccess = true;
          if (count < 5) {
            doTransaction1Put();
          }
        });
      }());

      transaction1.oncomplete = t.step_func(evt => {
        transaction1Complete = true;
        assert_false(
          transaction2PutSuccess,
          'transaction1 should complete before transaction2 put succeeds');
      });

      const request = transaction2.objectStore('store').put(2, 0);
      request.onerror = t.unreached_func('request should succeed');
      request.onsuccess = t.step_func(evt => {
        transaction2PutSuccess = true;
        assert_true(
          transaction1Complete,
          'transaction2 put should not succeed before transaction1 completes');
      });

      transaction2.oncomplete = t.step_func_done(evt => {
        assert_true(
          transaction1PutSuccess,
          'transaction1 put should succeed before transaction2 runs');
        assert_true(
          transaction1Complete,
          'transaction1 should complete before transaction2 runs');
        assert_true(
          transaction2PutSuccess,
          'transaction2 put should succeed before transaction2 completes');
      });
    });
  },
  "Check that readwrite transactions with overlapping scopes do not run in parallel.");
