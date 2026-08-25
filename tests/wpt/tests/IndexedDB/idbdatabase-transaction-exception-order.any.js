// META: global=window,worker
// META: title=IndexedDB: IDBDatabase transaction() Exception Ordering
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbdatabase-transaction

'use strict';

indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
    },
    (t, db) => {
      db.close();
      assert_throws_dom(
          'InvalidStateError', () => db.transaction('no-such-store'),
          '"Connection is closed" check (InvalidStateError) should precede ' +
              '"store names" check (NotFoundError)');
      t.done();
    },
    'IDBDatabase.transaction exception order: InvalidStateError vs. NotFoundError');

indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
    },
    (t, db) => {
      db.close();
      assert_throws_dom(
          'InvalidStateError', () => db.transaction([]),
          '"Connection is closed" check (InvalidStateError) should precede ' +
              '"stores is empty" check (InvalidAccessError)');
      t.done();
    },
    'IDBDatabase.transaction exception order: InvalidStateError vs. InvalidAccessError');

// Verify that the invalid mode check actually throws an exception.
indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
    },
    (t, db) => {
      assert_throws_js(
          TypeError, () => db.transaction('s', 'versionchange'),
          '"invalid mode" check should throw TypeError');
      t.done();
    },
    'IDBDatabase.transaction throws exception on invalid mode');

indexeddb_test(
    (t, db) => {
      db.createObjectStore('s');
    },
    (t, db) => {
      assert_throws_dom(
          'NotFoundError',
          () => db.transaction('no-such-store', 'versionchange'),
          '"No such store" check (NotFoundError) should precede ' +
              '"invalid mode" check (TypeError)');
      t.done();
    },
    'IDBDatabase.transaction exception order: NotFoundError vs. TypeError');
