// META: global=window,worker
// META: title=IndexedDB: IDBDatabase createObjectStore() Exception Ordering
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbdatabase-createobjectstore

'use strict';

indexeddb_test(
    (t, db, txn, rq) => {
      db.createObjectStore('s');

      // Acknowledge the error, to prevent an error from firing in browsers
      // that implement that.
      rq.onerror = e => {
        e.preventDefault();
      };

      txn.onabort = () => {
        setTimeout(
            t.step_func(() => {
              assert_throws_dom(
                  'InvalidStateError', () => db.createObjectStore('s2'),
                  '"running an upgrade transaction" check (InvalidStateError) ' +
                      'should precede "not active" check (TransactionInactiveError)');

              t.done();
            }),
            0);
      };
      txn.abort();
    },
    (t, db) => {
      t.assert_unreached(
          'open operation is expected to fail due to the aborted transaction.');
    },
    'IDBDatabase.createObjectStore exception order: ' +
        'InvalidStateError vs. TransactionInactiveError',
    {upgrade_will_abort: true});

indexeddb_test(
    (t, db, txn) => {
      const store = db.createObjectStore('s');

      txn.abort();

      assert_throws_dom(
          'TransactionInactiveError',
          () => db.createObjectStore('s2', {keyPath: '-invalid-'}),
          '"not active" check (TransactionInactiveError) should precede ' +
              '"valid key path" check (SyntaxError)');

      t.done();
    },
    (t, db) => {
      t.assert_unreached('open should fail');
    },
    'IDBDatabase.createObjectStore exception order: ' +
        'TransactionInactiveError vs. SyntaxError',
    {upgrade_will_abort: true});

indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
      assert_throws_dom(
          'SyntaxError',
          () => db.createObjectStore('s', {keyPath: 'not a valid key path'}),
          '"Invalid key path" check (SyntaxError) should precede ' +
              '"duplicate store name" check (ConstraintError)');
      t.done();
    },
    (t, db) => {},
    'IDBDatabase.createObjectStore exception order: ' +
        'SyntaxError vs. ConstraintError');

indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
      assert_throws_dom(
          'ConstraintError',
          () => db.createObjectStore('s', {autoIncrement: true, keyPath: ''}),
          '"already exists" check (ConstraintError) should precede ' +
              '"autoIncrement vs. keyPath" check (InvalidAccessError)');
      t.done();
    },
    (t, db) => {},
    'IDBDatabase.createObjectStore exception order: ' +
        'ConstraintError vs. InvalidAccessError');
