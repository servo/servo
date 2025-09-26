// META: global=window,worker
// META: title=IndexedDB: assure no crash when populating index
// META: script=../resources/support.js
// See https://crbug.com/434115938 for additional context and credits.

'use_strict';

indexeddb_test(
  (t, db, tx) => {
    const store = db.createObjectStore('store', { keyPath: 'a.b', autoIncrement: true });
    store.put({});
    const index = store.createIndex('index', 'keypath');
    t.done();
  },
  /*open_func=*/null,
  "Assure no crash when populating index",
);
