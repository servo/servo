// META: global=window,worker
// META: title=IDBCursor.advance()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Odin Hørthe Omdal <mailto:odinho@opera.com>
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
  const records = [{ pKey: "primaryKey_0", iKey: "indexKey_0" },
  { pKey: "primaryKey_1", iKey: "indexKey_1" },
  { pKey: "primaryKey_2", iKey: "indexKey_2" },
  { pKey: "primaryKey_3", iKey: "indexKey_3" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly")
      .objectStore("test")
      .index("index")
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
          let record = cursor.value;
          assert_equals(record.pKey, records[count].pKey, "record.pKey");
          assert_equals(record.iKey, records[count].iKey, "record.iKey");
          t.done();
          break;
        default:
          assert_unreached("unexpected count");
          break;
      }
    });
  };
}, "index - iterate cursor number of times specified by count");

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
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor != null, "cursor exist");
      assert_throws_js(TypeError,
        function () { cursor.advance(-1); });

      t.done();
    });
  };
}, "attempt to pass a count parameter that is not a number");

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
      .index("index")
      .openCursor(undefined, "next");

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      assert_true(cursor != null, "cursor exist");
      assert_throws_js(TypeError,
        function () { cursor.advance(-1); });

      t.done();
    });
  };
}, "index - attempt to advance backwards");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" },
    { pKey: "primaryKey_1-2", iKey: "indexKey_1" }
  ];
  const expected = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1-2", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly")
      .objectStore("test")
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, expected.length, "cursor run count")
        t.done()
      }

      let record = cursor.value;
      assert_equals(record.pKey, expected[count].pKey, "primary key");
      assert_equals(record.iKey, expected[count].iKey, "index key");

      cursor.advance(2);
      count++;
    });
  };
}, "index - iterate to the next record");

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
      assert_true(cursor instanceof IDBCursor);
      assert_throws_js(TypeError,
        function () { cursor.advance(0); });

      t.done();
    });
  }
}, "Calling advance() with count argument 0 should throw TypeError.");

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
      assert_true(cursor instanceof IDBCursor);

      event.target.transaction.abort();
      assert_throws_dom("TransactionInactiveError",
        function () { cursor.advance(1); });

      t.done();
    });
  }
}, "Calling advance() should throws an exception TransactionInactiveError when the transaction is not active.");

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
        function () { cursor.advance(1); });

      t.done();
    });
  }
}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");
