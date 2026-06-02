// META: global=window,worker
// META: title=IndexedDB: IDBCursor delete() Exception Ordering
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbcursor-delete

'use strict';

indexeddb_test(
    (t, db) => {
      const s = db.createObjectStore('s');
      s.put('value', 'key');
    },
    (t, db) => {
      const s = db.transaction('s', 'readonly').objectStore('s');
      const r = s.openCursor();
      r.onsuccess = t.step_func(() => {
        r.onsuccess = null;
        const cursor = r.result;
        setTimeout(
            t.step_func(() => {
              assert_throws_dom(
                  'TransactionInactiveError', () => cursor.delete(),
                  '"Transaction inactive" check (TransactionInactiveError)' +
                      'should precede "read only" check(ReadOnlyError)');
              t.done();
            }),
            0);
      });
    },
    'IDBCursor.delete exception order: TransactionInactiveError vs. ReadOnlyError');

indexeddb_test(
    (t, db) => {
      const s = db.createObjectStore('s');
      s.put('value', 'key');
    },
    (t, db) => {
      const s = db.transaction('s', 'readonly').objectStore('s');
      const r = s.openCursor();
      r.onsuccess = t.step_func(() => {
        r.onsuccess = null;
        const cursor = r.result;
        cursor.continue();
        assert_throws_dom(
            'ReadOnlyError', () => cursor.delete(),
            '"Read only" check (ReadOnlyError) should precede ' +
                '"got value flag" (InvalidStateError) check');
        t.done();
      });
    },
    'IDBCursor.delete exception order: ReadOnlyError vs. InvalidStateError #1');

indexeddb_test(
    (t, db) => {
      const s = db.createObjectStore('s');
      s.put('value', 'key');
    },
    (t, db) => {
      const s = db.transaction('s', 'readonly').objectStore('s');
      const r = s.openKeyCursor();
      r.onsuccess = t.step_func(() => {
        r.onsuccess = null;
        const cursor = r.result;
        assert_throws_dom(
            'ReadOnlyError', () => cursor.delete(),
            '"Read only" check (ReadOnlyError) should precede ' +
                '"key only flag" (InvalidStateError) check');
        t.done();
      });
    },
    'IDBCursor.delete exception order: ReadOnlyError vs. InvalidStateError #2');
