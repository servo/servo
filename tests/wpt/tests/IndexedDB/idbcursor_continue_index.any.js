// META: global=window,worker
// META: title=IDBCursor.continue() - index
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
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" },
    { pKey: "primaryKey_1-2", iKey: "indexKey_1" }
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
  };
}, "Iterate to the next record");

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

      assert_throws_dom("DataError",
        function () { cursor.continue(-1); });

      assert_true(cursor instanceof IDBCursorWithValue, "cursor");

      t.done();
    });
  };
}, "Attempt to pass a key parameter that is not a valid key");

async_test(t => {
  let dbObj = {};
  let count = 0;
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
      .openCursor(undefined, "next"); // XXX: Fx has issue with "undefined"

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, 2, "ran number of times");
        t.done();
      }

      // First time checks key equal, second time checks key less than
      assert_throws_dom("DataError",
        function () { cursor.continue(records[0].iKey); });

      cursor.continue();

      count++;
    });
  };
}, "Attempt to iterate to the previous record when the direction is set for the next record");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" },
    { pKey: "primaryKey_2", iKey: "indexKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor(undefined, "prev"); // XXX Fx issues w undefined

    cursor_rq.onsuccess = t.step_func(function (e) {
      let cursor = e.target.result;
      let record = cursor.value;

      switch (count) {
        case 0:
          assert_equals(record.pKey, records[2].pKey, "first pKey");
          assert_equals(record.iKey, records[2].iKey, "first iKey");
          cursor.continue();
          break;

        case 1:
          assert_equals(record.pKey, records[1].pKey, "second pKey");
          assert_equals(record.iKey, records[1].iKey, "second iKey");
          assert_throws_dom("DataError",
            function () { cursor.continue("indexKey_2"); });
          t.done();
          break;

        default:
          assert_unreached("Unexpected count value: " + count);
      }

      count++;
    });
  };
}, "Attempt to iterate to the next record when the direction is set for the previous record");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" },
    { pKey: "primaryKey_1-2", iKey: "indexKey_1" },
    { pKey: "primaryKey_2", iKey: "indexKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor(undefined, "prevunique");

    const expected = [
      { pKey: "primaryKey_2", iKey: "indexKey_2" },
      { pKey: "primaryKey_1", iKey: "indexKey_1" },
      { pKey: "primaryKey_0", iKey: "indexKey_0" }
    ];

    cursor_rq.onsuccess = t.step_func(function (e) {
      if (!e.target.result) {
        assert_equals(count, expected.length, 'count');
        t.done();
        return;
      }
      let cursor = e.target.result;
      let record = cursor.value;

      assert_equals(record.pKey, expected[count].pKey, "pKey #" + count);
      assert_equals(record.iKey, expected[count].iKey, "iKey #" + count);

      assert_equals(cursor.key, expected[count].iKey, "cursor.key #" + count);
      assert_equals(cursor.primaryKey, expected[count].pKey, "cursor.primaryKey #" + count);

      count++;
      cursor.continue(expected[count] ? expected[count].iKey : undefined);
    });
  };
}, "Iterate using 'prevunique'");

async_test(t => {
  let dbObj = {};
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", iKey: "indexKey_0" },
    { pKey: "primaryKey_1", iKey: "indexKey_1" },
    { pKey: "primaryKey_1-2", iKey: "indexKey_1" },
    { pKey: "primaryKey_2", iKey: "indexKey_2" }
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = setOnUpgradeNeeded(dbObj, records);

  open_rq.onsuccess = function (e) {
    let cursor_rq = dbObj.db.transaction("test", "readonly", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor(undefined, "nextunique");

    const expected = [
      { pKey: "primaryKey_0", iKey: "indexKey_0" },
      { pKey: "primaryKey_1", iKey: "indexKey_1" },
      { pKey: "primaryKey_2", iKey: "indexKey_2" }
    ];

    cursor_rq.onsuccess = t.step_func(function (e) {
      if (!e.target.result) {
        assert_equals(count, expected.length, 'count');
        t.done();
        return;
      }
      let cursor = e.target.result;
      let record = cursor.value;

      assert_equals(record.pKey, expected[count].pKey, "pKey #" + count);
      assert_equals(record.iKey, expected[count].iKey, "iKey #" + count);

      assert_equals(cursor.key, expected[count].iKey, "cursor.key #" + count);
      assert_equals(cursor.primaryKey, expected[count].pKey, "cursor.primaryKey #" + count);

      count++;
      cursor.continue(expected[count] ? expected[count].iKey : undefined);
    });
  };
}, "Iterate using nextunique");

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
        function () { cursor.continue(); });

      t.done();
    });
  }
}, "Calling continue() should throw an exception TransactionInactiveError when the transaction is not active.");

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
      assert_throws_dom("InvalidStateError",
        function () { cursor.continue(); });

      t.done();
    });
  }
}, "If the cursor's source or effective object store has been deleted, the implementation MUST throw a DOMException of type InvalidStateError");

async_test(t => {
  let db;
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", obj: { iKey: "iKey_0" } },
    { pKey: "primaryKey_1", obj: { iKey: "iKey_1" } },
    { pKey: "primaryKey_2", obj: { iKey: "iKey_2" } }
  ];

  const expected = [
    ["primaryKey_2", "iKey_2"],
    ["primaryKey_0", "iKey_0"]
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    var objStore = db.createObjectStore("test", { keyPath: ["pKey", "obj.iKey"] });
    objStore.createIndex("index", ["pKey", "obj.iKey"]);

    for (var i = 0; i < records.length; i++)
      objStore.add(records[i]);
  };

  open_rq.onsuccess = function (e) {
    var cursor_rq = db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor(null, "prev");

    cursor_rq.onsuccess = t.step_func(function (e) {
      var cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, 2, "cursor run count");
        t.done();
      }

      if (count === 0) {
        e.target.source.objectStore.delete(["primaryKey_1", "iKey_1"]);
      }
      assert_array_equals(cursor.key, expected[count], "primary key");

      cursor.continue();
      count++;
    });
  }
}, "Delete next element, and iterate to it");

async_test(t => {
  let db;
  let count = 0;
  const records = [
    { pKey: "primaryKey_0", obj: { iKey: "iKey_0" } },
    { pKey: "primaryKey_2", obj: { iKey: "iKey_2" } }
  ];

  const expected = [
    ["primaryKey_2", "iKey_2"],
    ["primaryKey_1", "iKey_1"],
    ["primaryKey_0", "iKey_0"]
  ];

  let open_rq = createdb(t);
  open_rq.onupgradeneeded = function (e) {
    db = e.target.result;
    var objStore = db.createObjectStore("test", { keyPath: "pKey" });
    objStore.createIndex("index", ["pKey", "obj.iKey"]);

    for (var i = 0; i < records.length; i++)
      objStore.add(records[i]);
  };

  open_rq.onsuccess = function (e) {
    var cursor_rq = db.transaction("test", "readwrite", { durability: 'relaxed' })
      .objectStore("test")
      .index("index")
      .openCursor(null, "prev");

    cursor_rq.onsuccess = t.step_func(function (e) {
      var cursor = e.target.result;
      if (!cursor) {
        assert_equals(count, 3, "cursor run count");
        t.done();
      }

      if (count === 0) {
        e.target.source.objectStore.add({ pKey: "primaryKey_1", obj: { iKey: "iKey_1" } });
      }
      assert_array_equals(cursor.key, expected[count], "primary key");

      cursor.continue();
      count++;
    });
  }
}, "Add next element, and iterate to it");
