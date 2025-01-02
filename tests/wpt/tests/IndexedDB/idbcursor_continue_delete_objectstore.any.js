// META: title=IDBObjectStore.delete() and IDBCursor.continue()
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  /* The goal here is to test that any prefetching of cursor values performs
   * correct invalidation of prefetched data.  This test is motivated by the
   * particularities of the Firefox implementation of preloading, and is
   * specifically motivated by an edge case when prefetching prefetches at
   * least 2 extra records and at most determines whether a mutation is
   * potentially relevant based on current cursor position and direction and
   * does not test for key equivalence.  Future implementations may want to
   * help refine this test if their cursors are more clever.
   *
   * Step-wise we:
   * - Open a cursor, returning key 0.
   * - When the cursor request completes, without yielding control:
   *   - Issue a delete() call that won't actually delete anything but looks
   *     relevant.  This should purge prefetched records 1 and 2.
   *   - Issue a continue() which should result in record 1 being fetched
   *     again and record 2 being prefetched again.
   *   - Delete record 2.  Unless there's a synchronously available source
   *     of truth, the data from continue() above will not be present and
   *     we'll expect the implementation to need to set a flag to invalidate
   *     the prefetched data when it arrives.
   * - When the cursor request completes, validate we got record 1 and issue
   *   a continue.
   * - When the request completes, we should have a null cursor result value
   *   because 2 was deleted.
   */
  let db;
  let count = 0;
  const records =
      [{pKey: 'primaryKey_0'}, {pKey: 'primaryKey_1'}, {pKey: 'primaryKey_2'}];

  // This is a key that is not present in the database, but that is known to
  // be relevant to a forward iteration of the above keys by comparing to be
  // greater than all of them.
  const plausibleFutureKey = 'primaryKey_9';

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;

    let objStore = db.createObjectStore('test', {keyPath: 'pKey'});

    for (let i = 0; i < records.length; i++)
      objStore.add(records[i]);
  };

  open_rq.onsuccess = t.step_func(CursorDeleteRecord);


  function CursorDeleteRecord(e) {
    let txn = db.transaction('test', 'readwrite', {durability: 'relaxed'});
    let object_store = txn.objectStore('test');
    let cursor_rq = object_store.openCursor();
    let iteration = 0;

    cursor_rq.onsuccess = t.step_func(function(e) {
      let cursor = e.target.result;

      switch (iteration) {
        case 0:
          object_store.delete(plausibleFutureKey);
          assert_true(cursor != null, 'cursor valid');
          assert_equals(cursor.value.pKey, records[iteration].pKey);
          cursor.continue();
          object_store.delete(records[2].pKey);
          break;
        case 1:
          assert_true(cursor != null, 'cursor valid');
          assert_equals(cursor.value.pKey, records[iteration].pKey);
          cursor.continue();
          break;
        case 2:
          assert_equals(cursor, null, 'cursor no longer valid');
          break;
      };
      iteration++;
    });

    txn.oncomplete = t.step_func(VerifyRecordWasDeleted);
  }


  function VerifyRecordWasDeleted(e) {
    let cursor_rq = db.transaction('test', 'readonly', {durability: 'relaxed'})
                        .objectStore('test')
                        .openCursor();

    cursor_rq.onsuccess = t.step_func(function(e) {
      let cursor = e.target.result;

      if (!cursor) {
        assert_equals(count, 2, 'count');
        t.done();
      }

      assert_equals(cursor.value.pKey, records[count].pKey);
      count++;
      cursor.continue();
    });
  }
}, 'Object store - remove a record from the object store while iterating cursor');
