// META: title=IDBRequest.result
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let open = createdb(t);
  open.onupgradeneeded = t.step_func(e => {
    let db = e.target.result;
    db.createObjectStore('store');
  });
  open.onsuccess = t.step_func(e => {
    let db = e.target.result;
    let request =
        db.transaction('store', 'readonly').objectStore('store').get(0);

    assert_equals(request.readyState, 'pending');
    assert_throws_dom(
        'InvalidStateError', () => request.result,
        'IDBRequest.result should throw if request is pending');
    t.done();
  });
}, 'IDBRequest.result throws if ready state is pending');
