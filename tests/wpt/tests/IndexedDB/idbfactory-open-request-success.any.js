// META: title=IDBFactory open(): request properties on success
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbfactory-open

'use strict';

let saw_complete = false;

indexeddb_test(
    (t, db, tx, rq) => {
      assert_equals(
          rq.readyState, 'done',
          'request done flag should be set during upgradeneeded');
      assert_equals(
          rq.result, db,
          'request result should be set (to connection) during upgradeneeded');
      assert_equals(
          rq.error, null, 'request result should be null during upgradeneeded');

      tx.onabort = t.unreached_func('transaction should complete');
      tx.oncomplete = t.step_func(() => {
        saw_complete = true;

        assert_equals(
            rq.readyState, 'done',
            'request done flag should still be set during complete');
        assert_equals(
            rq.result, db,
            'request result should still be set (to connection) during complete');
        assert_equals(
            rq.error, null,
            'request result should still be null during complete');
      });
    },
    (t, db, rq) => {
      assert_true(saw_complete, 'complete event should fire before success');
      assert_equals(
          rq.readyState, 'done', 'request done flag should be set on success');
      assert_equals(
          rq.result, db,
          'request result should still be set (to connection) on success');
      assert_equals(rq.error, null, 'request error should be null on success');
      t.done();
    },
    'Properties of IDBOpenDBRequest during successful IDBFactory open()');
