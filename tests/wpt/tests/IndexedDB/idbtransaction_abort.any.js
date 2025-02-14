// META: title=IDBTransaction - abort
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  let aborted;
  const record = {indexedProperty: 'bar'};

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    let txn = e.target.transaction;
    let objStore = db.createObjectStore('store');

    objStore.add(record, 1);
    objStore.add(record, 2);
    let index =
        objStore.createIndex('index', 'indexedProperty', {unique: true});

    assert_true(index instanceof IDBIndex, 'IDBIndex');

    e.target.transaction.onabort = t.step_func(function(e) {
      aborted = true;
      assert_equals(e.type, 'abort', 'event type');
    });

    db.onabort = function(e) {
      assert_true(aborted, 'transaction.abort event has fired');
      t.done();
    };

    e.target.transaction.oncomplete = fail(t, 'got complete, expected abort');
  };
});
