// META: script=support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store');
    db.createObjectStore('a');
    db.createObjectStore('b');
    db.createObjectStore('c');
  },

  (t, db) => {
    let transaction1Started = false;
    let transaction1Complete = false;
    let transaction2Started = false;
    let transaction2Complete = false;
    let transaction3Started = false;
    let transaction3Complete = false;

    const transaction1 = db.transaction(['a'], 'readwrite');
    let request = transaction1.objectStore('a').get(0);
    request.onerror = t.unreached_func('request should succeed');
    request.onsuccess = t.step_func(() => {
      transaction1Started = true;
    });
    transaction1.onabort = t.unreached_func('transaction1 should complete');
    transaction1.oncomplete = t.step_func(() => {
      transaction1Complete = true;
      assert_false(transaction2Started);
      assert_false(transaction3Started);
    });


    // transaction2 overlaps with transaction1, so must wait until transaction1
    // completes.
    const transaction2 = db.transaction(['a', 'b'], 'readwrite');
    request = transaction2.objectStore('a').get(0);
    request.onerror = t.unreached_func('request should succeed');
    request.onsuccess = t.step_func(() => {
      assert_true(transaction1Complete);
      transaction2Started = true;
    });
    transaction2.onabort = t.unreached_func('transaction2 should complete');
    transaction2.oncomplete = t.step_func(() => {
      transaction2Complete = true;
      assert_false(transaction3Started);
    });

    // transaction3 overlaps with transaction2, so must wait until transaction2
    // completes even though it does not overlap with transaction1.
    const transaction3 = db.transaction(['b', 'c'], 'readwrite');
    request = transaction3.objectStore('b').get(0);
    request.onerror = t.unreached_func('request should succeed');
    request.onsuccess = t.step_func(() => {
      assert_true(transaction1Complete);
      assert_true(transaction2Complete);
      transaction3Started = true;
    });
    transaction3.onabort = t.unreached_func('transaction3 should complete');
    transaction3.oncomplete = t.step_func_done(() => {
      transaction3Complete = true;
    });
  },
  "Check that scope restrictions on read-write transactions are enforced.");
