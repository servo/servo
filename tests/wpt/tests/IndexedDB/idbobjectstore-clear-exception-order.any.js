// META: global=window,worker
// META: title=IndexedDB: IDBObjectStore clear() Exception Ordering
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbobjectstore-clear

'use strict';

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('s');
      const store2 = db.createObjectStore('s2');

      db.deleteObjectStore('s2');

      setTimeout(
          t.step_func(() => {
            assert_throws_dom(
                'InvalidStateError',
                () => {
                  store2.clear();
                },
                '"has been deleted" check (InvalidStateError) should precede ' +
                    '"not active" check (TransactionInactiveError)');
            t.done();
          }),
          0);
    },
    (t, db) => {},
    'IDBObjectStore.clear exception order: ' +
        'InvalidStateError vs. TransactionInactiveError');

indexeddb_test(
    (t, db) => {
      const store = db.createObjectStore('s');
    },
    (t, db) => {
      const tx = db.transaction('s', 'readonly');
      const store = tx.objectStore('s');

      setTimeout(
          t.step_func(() => {
            assert_throws_dom(
                'TransactionInactiveError',
                () => {
                  store.clear();
                },
                '"not active" check (TransactionInactiveError) should precede ' +
                    '"read only" check (ReadOnlyError)');
            t.done();
          }),
          0);
    },

    'IDBObjectStore.clear exception order: ' +
        'TransactionInactiveError vs. ReadOnlyError');
