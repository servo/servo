// META: title=IndexedDB: IDBIndex keyPath attribute - same object
// META: script=support.js

indexeddb_test(
  (t, db) => {
    const store = db.createObjectStore('store', {keyPath: ['a', 'b']});
    store.createIndex('index', ['a', 'b']);
  },
  (t, db) => {
    const tx = db.transaction('store');
    const store = tx.objectStore('store');
    const index = store.index('index');
    assert_equals(typeof index.keyPath, 'object', 'keyPath is an object');
    assert_true(Array.isArray(index.keyPath), 'keyPath is an array');

    assert_equals(
      index.keyPath, index.keyPath,
      'Same object instance is returned each time keyPath is inspected');

    const tx2 = db.transaction('store');
    const store2 = tx2.objectStore('store');
    const index2 = store2.index('index');

    assert_not_equals(
      index.keyPath, index2.keyPath,
      'Different instances are returned from different index instances.');

    t.done();
  },
  `IDBIndex's keyPath attribute returns the same object.`);
