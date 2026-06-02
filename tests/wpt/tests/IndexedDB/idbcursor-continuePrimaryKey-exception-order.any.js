// META: global=window,worker
// META: title=IDBCursor.continuePrimaryKey() - Exception Orders
// META: script=resources/support.js
// META: timeout=long

// Spec: http://w3c.github.io/IndexedDB/#dom-idbcursor-continueprimarykey

'use strict';

function setup_test_store(db) {
  const records = [
    {iKey: 'A', pKey: 1}, {iKey: 'A', pKey: 2}, {iKey: 'A', pKey: 3},
    {iKey: 'A', pKey: 4}, {iKey: 'B', pKey: 5}, {iKey: 'B', pKey: 6},
    {iKey: 'B', pKey: 7}, {iKey: 'C', pKey: 8}, {iKey: 'C', pKey: 9},
    {iKey: 'D', pKey: 10}
  ];

  const store = db.createObjectStore('test', {keyPath: 'pKey'});
  store.createIndex('idx', 'iKey');

  for (let i = 0; i < records.length; i++) {
    store.add(records[i]);
  }

  return store;
}

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq = store.index('idx').openCursor();
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    cursor = e.target.result;
    assert_true(!!cursor, 'acquire cursor');

    store.deleteIndex('idx');
  });
  txn.oncomplete = t.step_func(() => {
    assert_throws_dom('TransactionInactiveError', () => {
      cursor.continuePrimaryKey('A', 4);
    }, 'transaction-state check should precede deletion check');
    t.done();
  });
}, null, 'TransactionInactiveError v.s. InvalidStateError(deleted index)');

indexeddb_test(
    function(t, db, txn) {
      const store = setup_test_store(db);
      const cursor_rq = store.openCursor();
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        cursor = e.target.result;
        assert_true(!!cursor, 'acquire cursor');

        db.deleteObjectStore('test');

        assert_throws_dom('InvalidStateError', () => {
          cursor.continuePrimaryKey('A', 4);
        }, 'deletion check should precede index source check');
        t.done();
      });
    },
    null,
    'InvalidStateError(deleted source) v.s. InvalidAccessError(incorrect source)');

indexeddb_test(
    function(t, db, txn) {
      const store = setup_test_store(db);
      const cursor_rq = store.index('idx').openCursor(null, 'nextunique');
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        cursor = e.target.result;
        assert_true(!!cursor, 'acquire cursor');

        store.deleteIndex('idx');

        assert_throws_dom('InvalidStateError', () => {
          cursor.continuePrimaryKey('A', 4);
        }, 'deletion check should precede cursor direction check');
        t.done();
      });
    },
    null,
    'InvalidStateError(deleted source) v.s. InvalidAccessError(incorrect direction)');

indexeddb_test(
    function(t, db, txn) {
      const store = db.createObjectStore('test', {keyPath: 'pKey'});

      store.add({iKey: 'A', pKey: 1});

      const cursor_rq =
          store.createIndex('idx', 'iKey').openCursor(null, 'nextunique');
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        if (e.target.result) {
          cursor = e.target.result;
          cursor.continue();
          return;
        }

        assert_throws_dom('InvalidAccessError', () => {
          cursor.continuePrimaryKey('A', 4);
        }, 'direction check should precede got_value_flag check');
        t.done();
      });
    },
    null,
    'InvalidAccessError(incorrect direction) v.s. InvalidStateError(iteration complete)');

indexeddb_test(
    function(t, db, txn) {
      const store = db.createObjectStore('test', {keyPath: 'pKey'});

      store.add({iKey: 'A', pKey: 1});

      const cursor_rq =
          store.createIndex('idx', 'iKey').openCursor(null, 'nextunique');
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        if (!cursor) {
          cursor = e.target.result;
          assert_true(!!cursor, 'acquire cursor');

          cursor.continue();

          assert_throws_dom('InvalidAccessError', () => {
            cursor.continuePrimaryKey('A', 4);
          }, 'direction check should precede iteration ongoing check');
          t.done();
        }
      });
    },
    null,
    'InvalidAccessError(incorrect direction) v.s. InvalidStateError(iteration ongoing)');

indexeddb_test(
    function(t, db, txn) {
      const store = setup_test_store(db);
      const cursor_rq = store.openCursor();
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        if (!cursor) {
          cursor = e.target.result;
          assert_true(!!cursor, 'acquire cursor');

          cursor.continue();

          assert_throws_dom('InvalidAccessError', () => {
            cursor.continuePrimaryKey('A', 4);
          }, 'index source check should precede iteration ongoing check');
          t.done();
        }
      });
    },
    null,
    'InvalidAccessError(incorrect source) v.s. InvalidStateError(iteration ongoing)');

indexeddb_test(
    function(t, db, txn) {
      const store = db.createObjectStore('test', {keyPath: 'pKey'});

      store.add({iKey: 'A', pKey: 1});

      const cursor_rq = store.openCursor();
      let cursor;

      cursor_rq.onerror = t.unreached_func('openCursor should succeed');
      cursor_rq.onsuccess = t.step_func((e) => {
        if (e.target.result) {
          cursor = e.target.result;
          cursor.continue();
          return;
        }

        assert_throws_dom('InvalidAccessError', () => {
          cursor.continuePrimaryKey('A', 4);
        }, 'index source check should precede got_value_flag check');
        t.done();
      });
    },
    null,
    'InvalidAccessError(incorrect source) v.s. InvalidStateError(iteration complete)');

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq = store.index('idx').openCursor();
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    if (!cursor) {
      cursor = e.target.result;
      assert_true(!!cursor, 'acquire cursor');

      cursor.continue();

      assert_throws_dom('InvalidStateError', () => {
        cursor.continuePrimaryKey(null, 4);
      }, 'iteration ongoing check should precede unset key check');
      t.done();
    }
  });
}, null, 'InvalidStateError(iteration ongoing) v.s. DataError(unset key)');

indexeddb_test(function(t, db, txn) {
  const store = db.createObjectStore('test', {keyPath: 'pKey'});

  store.add({iKey: 'A', pKey: 1});

  const cursor_rq = store.createIndex('idx', 'iKey').openCursor();
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    if (e.target.result) {
      cursor = e.target.result;
      cursor.continue();
      return;
    }

    assert_throws_dom('InvalidStateError', () => {
      cursor.continuePrimaryKey(null, 4);
    }, 'got_value_flag check should precede unset key check');
    t.done();
  });
}, null, 'InvalidStateError(iteration complete) v.s. DataError(unset key)');

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq = store.index('idx').openCursor();
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    cursor = e.target.result;
    assert_true(!!cursor, 'acquire cursor');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey(null, 4);
    }, 'DataError is expected if key is unset.');
    t.done();
  });
}, null, 'DataError(unset key)');

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq = store.index('idx').openCursor();
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    cursor = e.target.result;
    assert_true(!!cursor, 'acquire cursor');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('A', null);
    }, 'DataError is expected if primary key is unset.');
    t.done();
  });
}, null, 'DataError(unset primary key)');

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq = store.index('idx').openCursor(IDBKeyRange.lowerBound('B'));
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func((e) => {
    cursor = e.target.result;
    assert_true(!!cursor, 'acquire cursor');

    assert_equals(cursor.key, 'B', 'expected key');
    assert_equals(cursor.primaryKey, 5, 'expected primary key');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('A', 6);
    }, 'DataError is expected if key is lower then current one.');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('B', 5);
    }, 'DataError is expected if primary key is equal to current one.');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('B', 4);
    }, 'DataError is expected if primary key is lower than current one.');

    t.done();
  });
}, null, 'DataError(keys are lower then current one) in \'next\' direction');

indexeddb_test(function(t, db, txn) {
  const store = setup_test_store(db);
  const cursor_rq =
      store.index('idx').openCursor(IDBKeyRange.upperBound('B'), 'prev');
  let cursor;

  cursor_rq.onerror = t.unreached_func('openCursor should succeed');
  cursor_rq.onsuccess = t.step_func(function(e) {
    cursor = e.target.result;
    assert_true(!!cursor, 'acquire cursor');

    assert_equals(cursor.key, 'B', 'expected key');
    assert_equals(cursor.primaryKey, 7, 'expected primary key');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('C', 6);
    }, 'DataError is expected if key is larger then current one.');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('B', 7);
    }, 'DataError is expected if primary key is equal to current one.');

    assert_throws_dom('DataError', () => {
      cursor.continuePrimaryKey('B', 8);
    }, 'DataError is expected if primary key is larger than current one.');

    t.done();
  });
}, null, 'DataError(keys are larger then current one) in \'prev\' direction');
