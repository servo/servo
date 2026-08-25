// META: title=IndexedDB Transaction Request Queue Handling
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  let keys = {txn: [], txn2: []};
  let open_rq = createdb(t);

  open_rq.onupgradeneeded = function(e) {
    let i;
    let os;
    db = e.target.result;

    for (i = 1; i < 6; i++) {
      os = db.createObjectStore('os' + i, {autoIncrement: true, keyPath: 'k'});
      os.add({os: 'os' + i});
      os.put({os: 'os' + i, k: i});
      os.add({os: 'os' + i});
    }
  };

  open_rq.onsuccess = function(e) {
    let txn = db.transaction(['os2', 'os1', 'os3', 'os5'])
    txn.objectStore('os1').openCursor().onsuccess = reg('txn')
    txn.objectStore('os3').openCursor().onsuccess = reg('txn')
    txn.objectStore('os1').get(2).onsuccess = reg('txn')
    txn.objectStore('os2').get(3).onsuccess = reg('txn')

    let txn2 = db.transaction(['os4', 'os3', 'os1', 'os5'])
    let os4 = txn2.objectStore('os4')

    for (let i = 0; i < 3; i++) {
      os4.openCursor().onsuccess = reg('txn2')
      os4.get(5).onsuccess = reg('txn2')
      os4.get(4).onsuccess = reg('txn2')
      txn.objectStore('os2').get(1).onsuccess = reg('txn')
      txn2.objectStore('os3').get(1).onsuccess = reg('txn2')
    }

    txn2.objectStore('os1').get(2).onsuccess = reg('txn2')
    txn.objectStore('os1').openCursor(null, 'prev').onsuccess = reg('txn')
    os4.openCursor(null, 'prev').onsuccess = reg('txn2')

    txn.oncomplete = t.step_func(finish);
    txn2.oncomplete = t.step_func(finish);
  };


  function reg(n) {
    return t.step_func(function(e) {
      let v = e.target.result;
      if (v.value)
        v = v.value;
      keys[n].push(v.os + ': ' + v.k);
    });
  }

  let finish_to_go = 2;
  function finish() {
    if (--finish_to_go)
      return;

    assert_array_equals(
        keys['txn'],
        [
          'os1: 1',
          'os3: 1',
          'os1: 2',
          'os2: 3',
          'os2: 1',
          'os2: 1',
          'os2: 1',
          'os1: 2',
        ],
        'transaction keys');

    assert_array_equals(
        keys['txn2'],
        [
          'os4: 1',
          'os4: 5',
          'os4: 4',
          'os3: 1',
          'os4: 1',
          'os4: 5',
          'os4: 4',
          'os3: 1',
          'os4: 1',
          'os4: 5',
          'os4: 4',
          'os3: 1',
          'os1: 2',
          'os4: 5',
        ],
        'transaction 2 keys');

    t.done();
  }
}, 'Transactions have a request queue');
