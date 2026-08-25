// META: title=IndexedDB: open and delete request queues
// META: global=window,worker
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#connection-queues

'use strict';

async_test(t => {
  const db_name = 'db' + self.location.pathname + '-' + t.name;
  indexedDB.deleteDatabase(db_name);

  // Open and hold connection while other requests are queued up.
  const r = indexedDB.open(db_name, 1);
  r.onerror = t.unreached_func('open should succeed');
  r.onsuccess = t.step_func(e => {
    const db = r.result;

    const saw = expect(t, [
      'open1 success', 'open1 versionchange', 'delete1 blocked',
      'delete1 success', 'open2 success', 'open2 versionchange',
      'delete2 blocked', 'delete2 success'
    ]);

    function open(token, version) {
      const r = indexedDB.open(db_name, version);
      r.onsuccess = t.step_func(e => {
        saw(token + ' success');
        const db = r.result;
        db.onversionchange = t.step_func(e => {
          saw(token + ' versionchange');
          setTimeout(t.step_func(() => db.close()), 0);
        });
      });
      r.onblocked = t.step_func(e => saw(token + ' blocked'));
      r.onerror = t.unreached_func('open should succeed');
    }

    function deleteDatabase(token) {
      const r = indexedDB.deleteDatabase(db_name);
      r.onsuccess = t.step_func(e => saw(token + ' success'));
      r.onblocked = t.step_func(e => saw(token + ' blocked'));
      r.onerror = t.unreached_func('deleteDatabase should succeed');
    }

    open('open1', 2);
    deleteDatabase('delete1');
    open('open2', 3);
    deleteDatabase('delete2');

    // Now unblock the queue.
    db.close();
  });
}, 'IndexedDB: open and delete requests are processed as a FIFO queue');
