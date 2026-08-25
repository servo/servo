// META: global=window,worker
// META: title=IDBCursor.delete() - object store
// META: script=resources/support.js

'use strict';

function createObjectStoreAndPopulate(db, records) {
  let objStore = db.createObjectStore("test", { keyPath: "pKey" });

  for (let i = 0; i < records.length; i++) {
    objStore.add(records[i]);
  }
  return objStore;
}

function setOnUpgradeNeeded(dbObj, records) {
  return function (event) {
    dbObj.db = event.target.result;
    createObjectStoreAndPopulate(dbObj.db, records);
  };
}

async_test(t => {
  let dbObj = {}, count = 0;
  const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = t.step_func(CursorDeleteRecord);


  function CursorDeleteRecord(e) {
    let txn = dbObj.db.transaction("test", "readwrite");
    let cursor_rq = txn.objectStore("test").openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor != null, "cursor exist");
      cursor.delete();
    });

    txn.oncomplete = t.step_func(VerifyRecordWasDeleted);
  }


  function VerifyRecordWasDeleted(e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly")
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      if (!cursor) {
        assert_equals(count, 1, 'count');
        t.done();
      }

      assert_equals(cursor.value.pKey, records[1].pKey);
      count++;
      cursor.continue();
    });
  }

}, "Remove a record from the object store ");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly")
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor != null, "cursor exist");
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
    let objStore = createObjectStoreAndPopulate(db, records);
    let cursor_rq = objStore.openCursor();

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

}, "Index - attempt to remove a record in an inactive transaction");

async_test(t => {

  let db;
  const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    let objStore = createObjectStoreAndPopulate(db, records);
    let rq = objStore.openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");

      db.deleteObjectStore("test");
      assert_throws_dom("InvalidStateError", function () { cursor.delete(); });

      t.done();
    });
  }

}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");

async_test(t => {
  let dbObj = {};
  const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);
  open_rq.onsuccess = function (event) {
    let txn = dbObj.db.transaction("test", "readwrite");
    let rq = txn.objectStore("test").openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");

      cursor.continue();
      assert_throws_dom("InvalidStateError", function () { cursor.delete(); });

      t.done();
    });
  }
}, "Throw InvalidStateError when the cursor is being iterated");
