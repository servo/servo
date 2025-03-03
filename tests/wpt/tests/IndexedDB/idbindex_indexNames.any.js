// META: global=window,worker
// META: title=IDBObjectStore.indexNames
// META: script=resources/support.js

'use_strict';

async_test(t => {
  let db;
  const open_rq = createdb(t);
  open_rq.onupgradeneeded = t.step_func(e => {
    db = e.target.result;
    const objStore = db.createObjectStore('test', {keyPath: 'key'});
    objStore.createIndex('index', 'data');

    assert_equals(objStore.indexNames[0], 'index', 'indexNames');
    assert_equals(objStore.indexNames.length, 1, 'indexNames.length');
  });

  open_rq.onsuccess = t.step_func(e => {
    const objStore = db.transaction('test', 'readonly').objectStore('test');

    assert_equals(objStore.indexNames[0], 'index', 'indexNames (second)');
    assert_equals(objStore.indexNames.length, 1, 'indexNames.length (second)');

    t.done();
  });
}, 'Verify IDBObjectStore.indexNames property');
