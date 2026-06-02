// META: title=IndexedDB Transaction Creation During Version Change
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  let events = [];
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    db = e.target.result

    db.createObjectStore('store')
        .add('versionchange1', 1)
        .addEventListener('success', log('versionchange_add.success'))

    assert_throws_dom('InvalidStateError', function() {
      db.transaction('store', 'readonly')
    })

    e.target.transaction.objectStore('store').count(2).addEventListener(
        'success', log('versionchange_count.success'))

    assert_throws_dom('InvalidStateError', function() {
      db.transaction('store', 'readwrite')
    })

    open_rq.transaction.objectStore('store')
        .add('versionchange2', 2)
        .addEventListener('success', log('versionchange_add2.success'))

    open_rq.transaction.oncomplete = function(e) {
      log('versionchange_txn.complete')(e)

      db.transaction('store', 'readonly')
          .objectStore('store')
          .count()
          .addEventListener('success', log('complete_count.success'))
    }
  };

  open_rq.onsuccess = function(e) {
    log('open_rq.success')(e)

    let txn = db.transaction('store', 'readwrite')
    txn.objectStore('store').put('woo', 1).addEventListener(
        'success', log('complete2_get.success'))

    txn.oncomplete = t.step_func(function(e) {
      assert_array_equals(
          events,
          [
            'versionchange_add.success: 1',
            'versionchange_count.success: 0',
            'versionchange_add2.success: 2',
            'versionchange_txn.complete',
            'open_rq.success: [object IDBDatabase]',
            'complete_count.success: 2',
            'complete2_get.success: 1',
          ],
          'events')
      t.done()
    })
  };


  function log(msg) {
    return function(e) {
      if (e && e.target && e.target.error)
        events.push(msg + ': ' + e.target.error.name)
        else if (e && e.target && e.target.result !== undefined)
        events.push(msg + ': ' + e.target.result)
        else events.push(msg)
    };
  }
}, 'Attempt to create new transactions inside a versionchange transaction');
