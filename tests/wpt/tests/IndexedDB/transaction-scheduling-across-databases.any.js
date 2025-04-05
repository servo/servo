// META: script=resources/support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store');
  },

  (t, db1) => {
    // Open a second database.
    const db2name = db1.name + '-2';
    const delete_request = indexedDB.deleteDatabase(db2name);
    delete_request.onerror = t.unreached_func('deleteDatabase() should succeed');
    const open_request = indexedDB.open(db2name, 1);
    open_request.onerror = t.unreached_func('open() should succeed');
    open_request.onupgradeneeded = t.step_func(() => {
      const db2 = open_request.result;
      const store = db2.createObjectStore('store');
    });
    open_request.onsuccess = t.step_func(() => {
      const db2 = open_request.result;
      t.add_cleanup(() => {
        db2.close();
        indexedDB.deleteDatabase(db2.name);
      });

      let transaction1PutSuccess = false;
      let transaction2PutSuccess = false;

      const onTransactionComplete = barrier_func(2, t.step_func_done(() => {
        assert_true(transaction1PutSuccess,
                    'transaction1 should have executed at least one request');
        assert_true(transaction2PutSuccess,
                    'transaction1 should have executed at least one request');
      }));


      const transaction1 = db1.transaction('store', 'readwrite');
      transaction1.onabort = t.unreached_func('transaction1 should complete');
      transaction1.oncomplete = t.step_func(onTransactionComplete);

      const transaction2 = db2.transaction('store', 'readwrite');
      transaction2.onabort = t.unreached_func('transaction2 should complete');
      transaction2.oncomplete = t.step_func(onTransactionComplete);

      // Keep both transactions alive until each has reported at least one
      // successful operation.

      function doTransaction1Put() {
        const request = transaction1.objectStore('store').put(0, 0);
        request.onerror = t.unreached_func('put request should succeed');
        request.onsuccess = t.step_func(() => {
          transaction1PutSuccess = true;
          if (!transaction2PutSuccess)
            doTransaction1Put();
        });
      }

      function doTransaction2Put() {
        const request = transaction2.objectStore('store').put(0, 0);
        request.onerror = t.unreached_func('put request should succeed');
        request.onsuccess = t.step_func(() => {
          transaction2PutSuccess = true;
          if (!transaction1PutSuccess)
            doTransaction2Put();
        });
      }

      doTransaction1Put();
      doTransaction2Put();
    });
  },
  "Check that transactions in different databases can run in parallel.");
