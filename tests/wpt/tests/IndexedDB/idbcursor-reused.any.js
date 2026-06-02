// META: global=window,worker
// META: title=IDBCursor is reused
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbcursor-continue

'use strict';

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(e => {
    db = e.target.result;
    const os = db.createObjectStore('test');

    os.add('data', 'k');
    os.add('data2', 'k2');
  });

  open_rq.onsuccess = t.step_func(e => {
    let cursor;
    let count = 0;
    const rq =
        db.transaction('test', 'readonly').objectStore('test').openCursor();

    rq.onsuccess = t.step_func(e => {
      switch (count) {
        case 0:
          cursor = e.target.result;

          assert_equals(cursor.value, 'data', 'prerequisite cursor.value');
          cursor.custom_cursor_value = 1;
          e.target.custom_request_value = 2;

          cursor.continue();
          break;

        case 1:
          assert_equals(cursor.value, 'data2', 'prerequisite cursor.value');
          assert_equals(cursor.custom_cursor_value, 1, 'custom cursor value');
          assert_equals(
              e.target.custom_request_value, 2, 'custom request value');

          cursor.advance(1);
          break;

        case 2:
          assert_false(!!e.target.result, 'got cursor');
          assert_equals(cursor.custom_cursor_value, 1, 'custom cursor value');
          assert_equals(
              e.target.custom_request_value, 2, 'custom request value');
          break;
      }
      count++;
    });

    rq.transaction.oncomplete = t.step_func(e => {
      assert_equals(count, 3, 'cursor callback runs');
      assert_equals(
          rq.custom_request_value, 2, 'variable placed on old IDBRequest');
      assert_equals(
          cursor.custom_cursor_value, 1,
          'custom cursor value (transaction.complete)');
      t.done();
    });
  });
});
