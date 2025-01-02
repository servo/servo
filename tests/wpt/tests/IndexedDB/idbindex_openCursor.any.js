// META: global=window,worker
// META: title=IDBIndex.openCursor()
// META: script=resources/support.js

'use_strict';

async_test(t => {
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(e => {
    const db = e.target.result;
    const store = db.createObjectStore('store', {keyPath: 'key'});
    const index = store.createIndex('index', 'indexedProperty');

    store.add({key: 1, indexedProperty: 'data'});
    store.deleteIndex('index');

    assert_throws_dom('InvalidStateError', () => {
      index.openCursor();
    });
    t.done();
  });
}, 'If the index is deleted, throw InvalidStateError');

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(e => {
    db = e.target.result;
    const store = db.createObjectStore('store', {keyPath: 'key'});
    store.createIndex('index', 'indexedProperty');
    store.add({key: 1, indexedProperty: 'data'});
  });

  open_rq.onsuccess = t.step_func(e => {
    db = e.target.result;
    const tx = db.transaction('store', 'readonly', {durability: 'relaxed'});
    const index = tx.objectStore('store').index('index');
    tx.abort();

    assert_throws_dom('TransactionInactiveError', () => {
      index.openCursor();
    });
    t.done();
  });
}, 'If the transaction has been aborted, throw TransactionInactiveError');

async_test(t => {
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(e => {
    const db = e.target.result;
    const store = db.createObjectStore('store', {keyPath: 'key'});
    const index = store.createIndex('index', 'indexedProperty');
    store.add({key: 1, indexedProperty: 'data'});

    e.target.transaction.abort();

    assert_throws_dom('InvalidStateError', () => {
      index.openCursor();
    });
    t.done();
  });
}, 'If the index is deleted by an aborted upgrade, throw InvalidStateError');
