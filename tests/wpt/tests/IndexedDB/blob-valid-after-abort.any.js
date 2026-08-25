// META: title=Blob Valid After Abort
// META: global=window,worker
// META: script=resources/support.js
'use strict';

let key = "key";

indexeddb_test(
  function upgrade(t, db) {
    db.createObjectStore('store');
    },
    function success(t, db) {
      const blobAContent = 'Blob A content';
      const blobA = new Blob([blobAContent], { 'type': 'text/plain' });
      const value = { a0: blobA };

      const txn = db.transaction('store', 'readwrite');
      const store = txn.objectStore('store');

      store.put(value, key);
      const request = store.get(key);
      request.onsuccess = t.step_func(function () {
        readBlob = request.result.a0;
        txn.abort();
      });

      let readBlob;
      txn.onabort = () => {
        readBlob.text().then(
          t.step_func_done(text => assert_equals(text, blobAContent)),
          t.unreached_func());
      };
    },
  "A blob can be read back after the transaction that added it was aborted.");
