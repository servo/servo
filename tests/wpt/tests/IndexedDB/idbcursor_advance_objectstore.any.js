// META: global=window,worker
// META: title=IDBCursor.advance()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Intel <http://www.intel.com>

'use strict';

function createAndPopulateObjectStore(db, records) {
  let objStore = db.createObjectStore("store", { keyPath: "pKey" });
  for (let i = 0; i < records.length; i++) {
    objStore.add(records[i]);
  }
  return objStore;
}

function setOnUpgradeNeeded(dbObj, records) {
  return function (event) {
    dbObj.db = event.target.result;
    createAndPopulateObjectStore(dbObj.db, records);
  };
}

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" },
    { pKey: "primaryKey_2" },
    { pKey: "primaryKey_3" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("store", "readonly", { durability: 'relaxed' })
      .objectStore("store")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor);

      switch (count) {
        case 0:
          count += 3;
          cursor.advance(3);
          break;
        case 3:
          assert_equals(cursor.value.pKey, records[count].pKey, "cursor.value.pKey");
          t.done();
          break;
        default:
          assert_unreached("unexpected count");
          break;
      }
    });
  }
}, "object store - iterate cursor number of times specified by count");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (event) {
    let txn = dbObj.db.transaction("store", "readwrite", { durability: 'relaxed' });
    let rq = txn.objectStore("store").openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor);

      assert_throws_js(TypeError,
        function () { cursor.advance(0); });
      t.done();
    });
  }
},  "Calling advance() with count argument 0 should throw TypeError.");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (event) {
    let txn = dbObj.db.transaction("store", "readwrite", { durability: 'relaxed' });
    let rq = txn.objectStore("store").openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor);

      event.target.transaction.abort();
      assert_throws_dom("TransactionInactiveError",
        function () { cursor.advance(1); });
      t.done();
    });
  }
}, "Calling advance() should throws an exception TransactionInactiveError when the transaction is not active");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (event) {
    let txn = dbObj.db.transaction("store", "readwrite", { durability: 'relaxed' });
    let rq = txn.objectStore("store").openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor);

      cursor.advance(1);
      assert_throws_dom("InvalidStateError",
        function () { cursor.advance(1); });
      t.done();
    });
  }
}, "Calling advance() should throw DOMException when the cursor is currently being iterated.");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    let objStore = createAndPopulateObjectStore(db, records);
    let rq = objStore.openCursor();
    rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");

      db.deleteObjectStore("store");
      assert_throws_dom("InvalidStateError",
        function () { cursor.advance(1); });
      t.done();
    });
  }
}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");
