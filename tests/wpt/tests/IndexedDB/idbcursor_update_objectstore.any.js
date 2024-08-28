// META: global=window,worker
// META: title=IDBCursor.update() - object store
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
  let dbObj = {};
  const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);
  open_rq.onsuccess = CursorUpdateRecord;

  function CursorUpdateRecord(e) {
    let txn = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' }), cursor_rq = txn.objectStore("test")
      .openCursor();
    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      cursor.value.data = "New information!";
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

      assert_equals(cursor.value.data, "New information!");
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
    let objStore = createObjectStoreAndPopulate(db, records);
    let cursor_rq = objStore.openCursor();

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

}, "Object store - attempt to modify a record in an inactive transaction");

async_test(t => {
  let db;
  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    let objStore = db.createObjectStore("test");

    objStore.add("data", "key");
  };

  open_rq.onsuccess = t.step_func(function (e) {
    let txn = db.transaction("test", "readwrite", { durability: 'relaxed' });
    let cursor_rq = txn.objectStore("test").openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;

      let updatedValue = Object.assign({}, cursor.value);
      updatedValue.pKey = "new data!";
      cursor.update(updatedValue).onsuccess = t.step_func(function (e) {
        assert_equals(e.target.result, "key");
        t.done();
      });
    });
  });

}, "Index - modify a record in the object store ");

async_test(t => {

  let db;
  const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    let objStore = createObjectStoreAndPopulate(db, records);

    let cursor_rq = objStore.openCursor();

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exists");

      db.deleteObjectStore("test");

      let updatedValue = Object.assign({}, cursor.value);
      updatedValue.pKey += "_updated";
      assert_throws_dom("InvalidStateError",
        function () { cursor.update(updatedValue); });

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
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
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
    { pKey: "primaryKey_0", value: "value_0" },
    { pKey: "primaryKey_1", value: "value_1" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .openCursor();

    cursor_rq.onsuccess = t.step_func(function (event) {
      let cursor = event.target.result;
      assert_true(cursor instanceof IDBCursor, "cursor exists");

      cursor.continue();
      assert_throws_dom("InvalidStateError", function () {
        cursor.update({ pKey: "primaryKey_0", value: "value_0_updated" });
      });

      t.done();
    });
  }
}, "Throw InvalidStateError when the cursor is being iterated");
