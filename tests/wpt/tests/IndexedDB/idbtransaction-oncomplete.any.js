// META: title=IDBTransaction - complete event
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  let store;
  let open_rq = createdb(t);
  let stages = [];

  open_rq.onupgradeneeded = function(e) {
    stages.push('upgradeneeded');

    db = e.target.result;
    store = db.createObjectStore('store');

    e.target.transaction.oncomplete = function() {
      stages.push('complete');
    };
  };

  open_rq.onsuccess = function(e) {
    stages.push('success');

    let tx = db.transaction('store', 'readonly');
    store = tx.objectStore('store');
    store.openCursor().onsuccess =
        function(e) {
      stages.push('opencursor');
    }

    db.transaction('store', 'readonly')
        .objectStore('store')
        .count()
        .onsuccess = t.step_func(function(e) {
      assert_array_equals(stages, [
        'upgradeneeded', 'complete', 'success', 'opencursor'
      ]);
      t.done();
    });
  }
});
