// META: global=window,worker
// META: title=IDBCursor.continue() - object store
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

function setOnUpgradeNeededWithCleanup(t, dbObj, records) {
  return function (e) {
    dbObj.db = e.target.result;
    t.add_cleanup(function () {
      dbObj.db.close();
      indexedDB.deleteDatabase(dbObj.db.name);
    });
    createObjectStoreAndPopulate(dbObj.db, records);
  };
}

async_test(t => {
  let dbObj = {};
  let count = 0;

  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let store = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test");

    let cursor_rq = store.openCursor();
    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, records.length, "cursor run count");
        t.done();
      }

      let record = cursor.value;
      assert_equals(record.pKey, records[count].pKey, "primary key");
      assert_equals(record.iKey, records[count].iKey, "index key");

      cursor.continue();
      count++;
    });
  }
}, "Iterate to the next record");

async_test(t => {
  let dbObj = {};

  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test").openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor instanceof IDBCursor, "cursor exists");
      assert_throws_dom("DataError",
        function () { cursor.continue(-1); });

      t.done();
    });
  }

}, "Attempt to pass a key parameter is not a valid key");

async_test(t => {
  let dbObj = {};

  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor(undefined, "next");

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor instanceof IDBCursor, "cursor exist");
      assert_throws_dom("DataError",
        function () { cursor.continue(records[0].pKey); });

      t.done();
    });
  }
}, "Attempt to iterate to the previous record when the direction is set for the next record");

async_test(t => {
  let dbObj = {};

  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" },
    { pKey: "primaryKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let count = 0,
      cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
        .objectStore("test")
        .openCursor(null, "prev");

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor != null, "cursor exist");

      switch (count) {
        case 0:
          assert_equals(cursor.value.pKey, records[2].pKey, "first cursor pkey");
          cursor.continue(records[1].pKey);
          break;

        case 1:
          assert_equals(cursor.value.pKey, records[1].pKey, "second cursor pkey");
          assert_throws_dom("DataError",
            function () { cursor.continue(records[2].pKey); });
          t.done();
          break;

        default:
          assert_unreached("Unexpected count value: " + count);
      }

      count++;
    });
  }
}, "Attempt to iterate to the next record when the direction is set for the next record");

async_test(t => {
  let dbObj = {};

  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exists");

      e.target.transaction.abort();
      assert_throws_dom("TransactionInactiveError",
        function () { cursor.continue(); });

      t.done();
    });
  }

}, "Calling continue() should throws an exception TransactionInactiveError when the transaction is not active.");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    let objStore = createObjectStoreAndPopulate(db, records);

    let cursor_rq = objStore.openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exists");

      db.deleteObjectStore("test");
      assert_throws_dom("InvalidStateError",
        function () { cursor.continue(); });

      t.done();
    });
  }
}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" },
    { pKey: "primaryKey_2" }
  ];

  const expected_records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeededWithCleanup(t, dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, 2, "cursor run count");
        t.done();
      }

      let record = cursor.value;
      if (record.pKey == "primaryKey_0") {
        e.target.source.delete("primaryKey_1");
      }
      assert_equals(record.pKey, expected_records[count].pKey, "primary key");

      cursor.continue();
      count++;
    });
  }
}, "Delete next element, and iterate to it");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_2" }
  ];

  const expected_records = [
    { pKey: "primaryKey_0" },
    { pKey: "primaryKey_1" },
    { pKey: "primaryKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeededWithCleanup(t, dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, 3, "cursor run count");
        t.done();
      }

      let record = cursor.value;
      if (record.pKey == "primaryKey_0") {
        e.target.source.add({ pKey: "primaryKey_1" });
      }
      assert_equals(record.pKey, expected_records[count].pKey, "primary key");

      cursor.continue();
      count++;
    });
  }
}, "Add next element, and iterate to it");
