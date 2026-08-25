// META: title=IDBCursor.continue()
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let objStore = db.createObjectStore('test');

    objStore.createIndex('index', '');

    objStore.add('data', 1);
    objStore.add('data2', 2);
  };

  open_rq.onsuccess = function(e) {
    let count = 0;
    let cursor_rq = db.transaction('test', 'readonly')
                        .objectStore('test')
                        .index('index')
                        .openCursor();

    cursor_rq.onsuccess = t.step_func(function(e) {
      if (!e.target.result) {
        assert_equals(count, 2, 'count');
        t.done();
        return;
      }
      let cursor = e.target.result;

      cursor.continue(undefined);

      // Second try
      assert_throws_dom('InvalidStateError', function() {
        cursor.continue();
      }, 'second continue');

      assert_throws_dom('InvalidStateError', function() {
        cursor.continue(3);
      }, 'third continue');

      count++;
    });
  };
}, 'Attempt to call continue two times');
