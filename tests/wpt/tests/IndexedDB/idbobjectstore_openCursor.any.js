// META: global=window,worker
// META: title=IDBObjectStore.openCursor() - iterate through 100 objects
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  const open_rq = createdb(t);

  open_rq.onupgradeneeded = t.step_func(e => {
    db = e.target.result;
    let store = db.createObjectStore('store');

    for (let i = 0; i < 100; i++)
      store.add('record_' + i, i);
  });

  open_rq.onsuccess = t.step_func(e => {
    let count = 0;
    let txn = db.transaction('store', 'readonly');

    txn.objectStore('store').openCursor().onsuccess = t.step_func(function(e) {
      if (e.target.result) {
        count += 1;
        e.target.result.continue();
      }
    })

    txn.oncomplete = t.step_func(function() {
      assert_equals(count, 100);
      t.done();
    })
  });
});
