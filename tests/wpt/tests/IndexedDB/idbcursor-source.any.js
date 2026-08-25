// META: global=window,worker
// META: title=IDBCursor.source
// META: script=resources/support.js

'use strict';

function cursor_source_test(
    test_name, name, stringified_object, cursor_rq_func) {
  indexeddb_test(
      function(t, db, tx) {
        const objStore = db.createObjectStore('my_objectstore');
        objStore.createIndex('my_index', '');

        objStore.add('data', 1);
        objStore.add('data2', 2);
      },
      function(t, db) {
        const cursor_rq = cursor_rq_func(db);

        cursor_rq.onsuccess = t.step_func((e) => {
          if (!e.target.result) {
            return;
          }
          const cursor = e.target.result;
          assert_readonly(cursor, 'source');

          // Direct try
          assert_true(cursor.source instanceof Object, 'source isobject');
          assert_equals(cursor.source + '', stringified_object, 'source');
          assert_equals(cursor.source.name, name, 'name');

          cursor.continue();
        });

        cursor_rq.transaction.oncomplete = t.step_func((e) => {
          t.done();
        });

        cursor_rq.transaction.onerror = t.step_func((e) => {
          assert_unreached(
              'Transaction got error. ' +
              (e.target.error ? e.target.error.name : 'unknown'));
        });
      },
      test_name);
}

cursor_source_test(
    'IDBCursor.source - IDBObjectStore', 'my_objectstore',
    '[object IDBObjectStore]', function(db) {
      return db.transaction('my_objectstore', 'readonly')
          .objectStore('my_objectstore')
          .openCursor();
    });

cursor_source_test(
    'IDBCursor.source - IDBIndex', 'my_index', '[object IDBIndex]',
    function(db) {
      return db.transaction('my_objectstore', 'readonly')
          .objectStore('my_objectstore')
          .index('my_index')
          .openCursor();
    });
