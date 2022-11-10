// META: title=Blob Valid Before Commit
// META: script=resources/support.js

let key = "key";

indexeddb_test(
    function upgrade(t, db) {
        db.createObjectStore('store');
    },
    function success(t, db) {
      const blobAContent = "Blob A content";
      const blobBContent = "Blob B content";
      const blobA = new Blob([blobAContent], {"type" : "text/plain"});
      const blobB = new Blob([blobBContent], {"type" : "text/plain"});
      const value = { a0: blobA, a1: blobA, b0: blobB };

      const tx = db.transaction('store', 'readwrite', {durability: 'relaxed'});
      const store = tx.objectStore('store');

      store.put(value, key);
      const request = store.get(key);

      request.onsuccess = t.step_func(function() {
        const record = request.result;

        const promise1 = record.a0.text().then(t.step_func(text => { assert_equals(text, blobAContent); },
            t.unreached_func()));

        const promise2 = record.a1.text().then(t.step_func(text => { assert_equals(text, blobAContent); },
            t.unreached_func()));

        const promise3 = record.b0.text().then(t.step_func(text => { assert_equals(text, blobBContent); },
            t.unreached_func()));

        Promise.all([promise1, promise2, promise3]).then(function() {
          // The test passes if it successfully completes.
          t.done();
        });
      });
    },
    "Blobs can be read back before their records are committed.");
