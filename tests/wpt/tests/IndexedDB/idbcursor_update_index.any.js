// META: global=window,worker
// META: title=IDBCursor.update() - index
// META: script=resources/support.js

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
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = CursorUpdateRecord;


  function CursorUpdateRecord(e) {
    let txn = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' }), cursor_rq = txn.objectStore("test")
      .index("index")
      .openCursor();
    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      cursor.value.iKey += "_updated";
      cursor.update(cursor.value);
    });

    txn.oncomplete = t.step_func(VerifyRecordWasUpdated);
  }


  function VerifyRecordWasUpdated(e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_equals(cursor.value.iKey, records[0].iKey + "_updated");

      t.done();
    });
  }

}, "Modify a record in the object store ");

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
      assert_throws_dom('ReadOnlyError',
        function () { cursor.update(cursor.value); });

      t.done();
    });
  }

}, "Attempt to modify a record in a read-only transaction");

async_test(t => {
  let db;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    let objStore = db.createObjectStore("test", { keyPath: "pKey" });
    let index = objStore.createIndex("index", "iKey");

    for (let i = 0; i < records.length; i++)
      objStore.add(records[i]);

    let cursor_rq = index.openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exist");
      self.cursor = cursor;
      self.record = cursor.value;
    });

    e.target.transaction.oncomplete = t.step_func(function (e) {
      assert_throws_dom('TransactionInactiveError',
        function () { self.cursor.update(self.record); })

      t.done();
    });
  }

}, "Attempt to modify a record in an inactive transaction");

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

      db.deleteObjectStore("test");
      cursor.value.iKey += "_updated";
      assert_throws_dom("InvalidStateError",
        function () { cursor.update(cursor.value); });

      t.done();
    });
  }

}, "Attempt to modify a record after the cursor's source or effective object store has been deleted. The implementation MUST throw a DOMException of type InvalidStateError");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor);

      let record = cursor.value;
      record.data = self;
      assert_throws_dom('DataCloneError',
        function () { cursor.update(record); });

      t.done();
    });
  }
}, "Throw DataCloneError");

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
      assert_true(cursor instanceof IDBCursor);
      assert_throws_js(TypeError, function () { cursor.update(); });

      t.done();
    });
  }
}, "No argument");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);
  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor);
      assert_throws_dom('DataError', function () { cursor.update(null); });

      t.done();
    });
  }
}, "Throw DataError");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);
  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exists");

      cursor.continue();
      assert_throws_dom("InvalidStateError", function () {
        cursor.update({ pKey: "primaryKey_0", iKey: "indexKey_0_updated" });
      });

      t.done();
    });
  }
}, "Throw InvalidStateError when the cursor is being iterated");

async_test(t => {
  let dbObj = {};
  const records = [
    { pKey: "primaryKey_1", iKey: 1 },
    { pKey: "primaryKey_2", iKey: 2 },
    { pKey: "primaryKey_3", iKey: 3 },
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = t.step_func(ModifyRecordsInIteration);

  // Iterate and modify values during iteration
  function ModifyRecordsInIteration(e) {
    let txn = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' });
    let index = txn.objectStore("test").index("index");
    let cursor_rq = index.openCursor(IDBKeyRange.upperBound(9));

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      if (!cursor) {
        return;
      }

      // Modify the record's value during iteration
      let record = cursor.value;
      record.iKey += 1;
      cursor.update(record);

      cursor.continue();
    });

    txn.oncomplete = t.step_func(VerifyUpdatedRecords);
  }

  // Verify that the records were updated correctly
  function VerifyUpdatedRecords(e) {
    let txn = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' });
    let objectStore = txn.objectStore("test");
    let getAll_rq = objectStore.getAll();

    getAll_rq.onsuccess = t.step_func(function (e) {
      // All values should have been incremented to 10
      assert_array_equals(
        e.target.result.map(record => record.iKey),
        [10, 10, 10],
        'iKey values should all be incremented until bound reached');

      t.done();
    });
  }

}, "Modify records during cursor iteration and verify updated records");
