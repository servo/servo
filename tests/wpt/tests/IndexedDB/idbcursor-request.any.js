// META: script=resources/support.js

function cursorRequestTest({ useIndex, useKeyCursor }) {
  indexeddb_test(
    (t, db) => {
      const objStore = db.createObjectStore("my_objectstore");
      objStore.add("data",  1);
      objStore.createIndex("my_index", "");
    },
    (t, db) => {
      const tx = db.transaction("my_objectstore", "readonly");
      let source = tx.objectStore("my_objectstore");
      if (useIndex) source = source.index('my_index');
      const req = useKeyCursor ? source.openKeyCursor() : source.openCursor();
      let cursor;

      req.onsuccess = t.step_func(() => {
        cursor = req.result;
        assert_equals(cursor.request, req, 'cursor.request');
        assert_readonly(cursor, 'request');
        assert_equals(cursor.request, cursor.request, 'cursor.request does not change');
      });

      req.transaction.oncomplete = t.step_func(() => {
        setTimeout(t.step_func(() => {
          assert_equals(cursor.request, req, 'cursor.request after transaction complete');
          t.done();
        }), 0);
      });

      req.transaction.onerror = t.unreached_func('Transaction error');
    },
    `cursor.request from ${useIndex ? 'IDBIndex' : 'IDBObjectStore'}.${useKeyCursor ? 'openKeyCursor' : 'openCursor'}`
  );
}

for (const useIndex of [false, true]) {
  for (const useKeyCursor of [false, true]) {
    cursorRequestTest({ useIndex, useKeyCursor });
  }
}
