// META: title=IDBFactory deleteDatabase()
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#dom-idbfactory-deleteDatabase

'use strict';

async_test(t => {
  const dbname = self.location + '-' + t.name;
  const rq = indexedDB.deleteDatabase(dbname);
  rq.onerror = t.unreached_func('deleteDatabase should succeed');
  rq.onsuccess = t.step_func(() => {
    assert_equals(
        rq.readyState, 'done', 'request done flag should be set on success');
    assert_equals(
        rq.result, undefined,
        'request result should be undefined on success');
    assert_equals(rq.error, null, 'request error should be null on success');
    t.done();
  });
}, 'IDBFactory deleteDatabase() request properties on success');
