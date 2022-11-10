// META: script=resources/support-promises.js

promise_test(async t => {
  const db = await createDatabase(t, db => {
    const store = db.createObjectStore('store');
    store.createIndex('index', 'value');
    store.put({value: 1}, 1);
    store.put({value: 2}, 2);
    store.put({value: 3}, 3);
  });

  {
    // Iterate over all index entries until an upper bound is reached.
    // On each record found, increment the value used as the index
    // key, which will make it show again up later in the iteration.
    const tx = db.transaction('store', 'readwrite', {durability: 'relaxed'});
    const range = IDBKeyRange.upperBound(9);
    const index = tx.objectStore('store').index('index');
    const request = index.openCursor(range);
    request.onsuccess = t.step_func(e => {
      const cursor = e.target.result;
      if (!cursor)
        return;

      const record = cursor.value;
      record.value += 1;
      cursor.update(record);

      cursor.continue();
    });

    await promiseForTransaction(t, tx);
  }

  {
    const tx = db.transaction('store', 'readonly', {durability: 'relaxed'});
    const results = await promiseForRequest(t, tx.objectStore('store').getAll());
    assert_array_equals(
      results.map(record => record.value),
      [10, 10, 10],
      'Values should all be incremented until bound reached');
  }
}, 'Index cursor - indexed values updated during iteration');
