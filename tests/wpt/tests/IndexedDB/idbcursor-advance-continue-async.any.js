// META: global=window,worker
// META: title=IDBCursor asyncness
// META: script=resources/support.js

'use strict';

function upgrade_func(t, db, tx) {
  let objStore = db.createObjectStore('test');
  objStore.createIndex('index', '');

  objStore.add('data', 1);
  objStore.add('data2', 2);
}

indexeddb_test(upgrade_func, function(t, db) {
  let count = 0;
  let rq = db.transaction('test', 'readonly').objectStore('test').openCursor();

  rq.onsuccess = t.step_func(function(e) {
    if (!e.target.result) {
      assert_equals(count, 2, 'count');
      t.done();
      return;
    }
    let cursor = e.target.result;

    switch (count) {
      case 0:
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 1);
        cursor.advance(1);
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 1);
        break;

      case 1:
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 2);
        cursor.advance(1);
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 2);
        break;

      default:
        assert_unreached('Unexpected count: ' + count);
    }

    count++;
  });
  rq.onerror = t.unreached_func('unexpected error');
}, 'IDBCursor asyncness - advance');

indexeddb_test(upgrade_func, function(t, db) {
  let count = 0;
  let rq = db.transaction('test', 'readonly')
               .objectStore('test')
               .index('index')
               .openCursor();

  rq.onsuccess = t.step_func(function(e) {
    if (!e.target.result) {
      assert_equals(count, 2, 'count');
      t.done();
      return;
    }
    let cursor = e.target.result;

    switch (count) {
      case 0:
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 'data');
        assert_equals(cursor.primaryKey, 1);
        cursor.continue('data2');
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 'data');
        assert_equals(cursor.primaryKey, 1);
        break;

      case 1:
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 'data2');
        assert_equals(cursor.primaryKey, 2);
        cursor.continue();
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 'data2');
        assert_equals(cursor.primaryKey, 2);
        break;

      default:
        assert_unreached('Unexpected count: ' + count);
    }

    count++;
  });
  rq.onerror = t.unreached_func('unexpected error');
}, 'IDBCursor asyncness - continue');

indexeddb_test(upgrade_func, function(t, db) {
  let count = 0;
  let rq = db.transaction('test', 'readonly')
               .objectStore('test')
               .index('index')
               .openCursor();

  rq.onsuccess = t.step_func(function(e) {
    if (!e.target.result) {
      assert_equals(count, 2, 'count');
      t.done();
      return;
    }
    let cursor = e.target.result;
    cursor.advance(1);

    switch (count) {
      case 0:
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 'data');
        assert_equals(cursor.primaryKey, 1);
        break;

      case 1:
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 'data2');
        assert_equals(cursor.primaryKey, 2);
        break;

      default:
        assert_unreached('Unexpected count: ' + count);
    }

    count++;
  });
  rq.onerror = t.unreached_func('unexpected error');
}, 'IDBCursor asyncness - fresh advance still async');

indexeddb_test(upgrade_func, function(t, db) {
  let count = 0;
  let rq = db.transaction('test', 'readonly').objectStore('test').openCursor();

  rq.onsuccess = t.step_func(function(e) {
    if (!e.target.result) {
      assert_equals(count, 2, 'count');
      t.done();
      return;
    }
    let cursor = e.target.result;
    cursor.continue();

    switch (count) {
      case 0:
        assert_equals(cursor.value, 'data');
        assert_equals(cursor.key, 1);
        break;

      case 1:
        assert_equals(cursor.value, 'data2');
        assert_equals(cursor.key, 2);
        break;

      default:
        assert_unreached('Unexpected count: ' + count);
    }

    count++;
  });
  rq.onerror = t.unreached_func('unexpected error');
}, 'IDBCursor asyncness - fresh continue still async');
