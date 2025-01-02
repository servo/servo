// META: global=window,worker
// META: title=IDBCursor.delete() - index
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Intel <http://www.intel.com>

'use strict';

function createObjectStoreWithIndexAndPopulate(db, records) {
  let objStore = db.createObjectStore("test", { keyPath: "pKey" });
  objStore.createIndex("index", "iKey");
  for (let i = 0; i < records.length; i++) {
    objStore.add(records[i]);
  }
  return objStore;
}

function setOnUpgradeNeeded(dbObj, records) {
  return function (event) {
    dbObj.db = event.target.result;
    createObjectStoreWithIndexAndPopulate(dbObj.db, records);
  };
}

async_test(t => {
  let dbObj = {};
  let count = 0;

  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = t.step_func(CursorDeleteRecord);

  function CursorDeleteRecord(e) {
    let txn = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' }),
      cursor_rq = txn.objectStore("test")
        .index("index")
        .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor instanceof IDBCursor, "cursor exist");
      cursor.delete();
    });

    txn.oncomplete = t.step_func(VerifyRecordWasDeleted);
  }


  function VerifyRecordWasDeleted(e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      if (!cursor) {
        assert_equals(count, 1, 'count');
        t.done();
      }

      assert_equals(cursor.value.pKey, records[1].pKey);
      assert_equals(cursor.value.iKey, records[1].iKey);
      cursor.continue();
      count++;
    });
  }
}, "Remove a record from the object store");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor instanceof IDBCursor, "cursor exist");
      assert_throws_dom('ReadOnlyError', function () { cursor.delete(); });
      t.done();
    });
  }
}, "Attempt to remove a record in a read-only transaction");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    let objStore = createObjectStoreWithIndexAndPopulate(db, records);

    let cursor_rq = objStore.index("index").openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");
      self.cursor = cursor;
    });

    e.target.transaction.oncomplete = t.step_func(function (e) {
      assert_throws_dom('TransactionInactiveError',
        function () { self.cursor.delete(); })
      t.done();
    });
  }
}, "Attempt to remove a record in an inactive transaction");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    let objStore = createObjectStoreWithIndexAndPopulate(db, records);

    let rq = objStore.index("index").openCursor();

    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");

      db.deleteObjectStore("test");
      assert_throws_dom("InvalidStateError",
        function () { cursor.delete(); });

      t.done();
    });
  }
}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    let objStore = createObjectStoreWithIndexAndPopulate(db, records);
    let rq = objStore.index("index").openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");

      cursor.continue();
      assert_throws_dom("InvalidStateError", function () {
        cursor.delete();
      });

      t.done();
    });
  }
}, "Throw InvalidStateError when the cursor is being iterated");
