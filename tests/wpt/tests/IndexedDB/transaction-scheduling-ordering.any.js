// META: script=resources/support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store');
  },

  (t, db) => {
    // Create in order tx1, tx2.
    const tx1 = db.transaction('store', 'readwrite');
    const tx2 = db.transaction('store', 'readwrite');

    // Use in order tx2, tx1.
    tx2.objectStore('store').get(0);
    tx1.objectStore('store').get(0);

    const order = [];
    const done = barrier_func(2, t.step_func_done(() => {
      // IndexedDB Spec:
      // https://w3c.github.io/IndexedDB/#transaction-scheduling
      //
      // If multiple "readwrite" transactions are attempting to
      // access the same object store (i.e. if they have overlapping
      // scope), the transaction that was created first must be the
      // transaction which gets access to the object store first.
      //
      assert_array_equals(order, [1, 2]);
    }));

    tx1.oncomplete = t.step_func(e => {
      order.push(1);
      done();
    });

    tx2.oncomplete = t.step_func(e => {
      order.push(2);
      done();
    });
  },
  "Verify Indexed DB transactions are ordered per spec");
