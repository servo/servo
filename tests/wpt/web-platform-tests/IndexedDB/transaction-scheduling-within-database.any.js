// META: script=support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store');
    store.put('value', 'key');
  },

  (t, db) => {
    let transaction1GetSuccess = false;
    let transaction2GetSuccess = false;

    const onTransactionComplete = barrier_func(2, t.step_func_done(() => {
      assert_true(transaction1GetSuccess,
                  'transaction1 should have executed at least one request');
      assert_true(transaction2GetSuccess,
                  'transaction1 should have executed at least one request');
    }));

    const transaction1 = db.transaction('store', 'readonly');
    transaction1.onabort = t.unreached_func('transaction1 should not abort');
    transaction1.oncomplete = t.step_func(onTransactionComplete);

    const transaction2 = db.transaction('store', 'readonly');
    transaction2.onabort = t.unreached_func('transaction2 should not abort');
    transaction2.oncomplete = t.step_func(onTransactionComplete);

    // Keep both transactions alive until each has reported at least one
    // successful operation

    function doTransaction1Get() {
      const request = transaction1.objectStore('store').get('key');
      request.onerror = t.unreached_func('request should not fail');
      request.onsuccess = t.step_func(() => {
        transaction1GetSuccess = true;
        if (!transaction2GetSuccess)
          doTransaction1Get();
      });
    }

    function doTransaction2Get() {
      // NOTE: No logging since execution order is not deterministic.
      const request = transaction2.objectStore('store').get('key');
      request.onerror = t.unreached_func('request should not fail');
      request.onsuccess = t.step_func(() => {
        transaction2GetSuccess = true;
        if (!transaction1GetSuccess)
          doTransaction2Get();
      });
    }

    doTransaction1Get();
    doTransaction2Get();
  },
  'Check that read-only transactions within a database can run in parallel.');
