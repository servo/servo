// META: global=window,worker
// META: title=IDBObjectStore.add()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Intel <http://www.intel.com>

'use_strict';

async_test(t => {
    let db;
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "key" });

      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .get(record.key);

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.property, record.property);
        assert_equals(e.target.result.key, record.key);
        t.done();
      });
    };
}, 'add() with an inline key');

async_test(t => {
    let db;
    const key = 1;
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store");

      objStore.add(record, key);
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .get(key);

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.property, record.property);

        t.done();
      });
    };
}, 'add() with an out-of-line key');

async_test(t => {
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "key" });
      objStore.add(record);

      const rq = objStore.add(record);
      rq.onsuccess = fail(t, "success on adding duplicate record");

      rq.onerror = t.step_func(function(e) {
        assert_equals(e.target.error.name, "ConstraintError");
        assert_equals(rq.error.name, "ConstraintError");
        assert_equals(e.type, "error");

        e.preventDefault();
        e.stopPropagation();
      });
    };

    // Defer done, giving rq.onsuccess a chance to run
    open_rq.onsuccess = function(e) {
      t.done();
    };
}, 'add() record with same key already exists');

async_test(t => {
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;
      const objStore = db.createObjectStore("store", { autoIncrement: true });
      objStore.createIndex("i1", "property", { unique: true });
      objStore.add(record);

      const rq = objStore.add(record);
      rq.onsuccess = fail(t, "success on adding duplicate indexed record");

      rq.onerror = t.step_func(function(e) {
        assert_equals(rq.error.name, "ConstraintError");
        assert_equals(e.target.error.name, "ConstraintError");
        assert_equals(e.type, "error");

        e.preventDefault();
        e.stopPropagation();
      });
    };

    // Defer done, giving a spurious rq.onsuccess a chance to run
    open_rq.onsuccess = function(e) {
      t.done();
    };
}, 'add() where an index has unique:true specified');

async_test(t => {
    let db;
    const record = { test: { obj: { key: 1 } }, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store",
        { keyPath: "test.obj.key" });
      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .get(record.test.obj.key);

      rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.property, record.property);

        t.done();
      });
    };
}, 'add() object store\'s key path is an object attribute');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "key",
       autoIncrement: true });

      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      const actual_keys = [];
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .openCursor();

      rq.onsuccess = t.step_func(function(e) {
        const cursor = e.target.result;

        if (cursor) {
          actual_keys.push(cursor.value.key);
          cursor.continue();
        } else {
          assert_array_equals(actual_keys, expected_keys);
          t.done();
        }
      });
    };
}, 'add() autoIncrement and inline keys');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store", { autoIncrement: true });

      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      const actual_keys = [];
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .openCursor();

      rq.onsuccess = t.step_func(function(e) {
        const cursor = e.target.result;

        if (cursor) {
          actual_keys.push(cursor.key);
          cursor.continue();
        } else {
          assert_array_equals(actual_keys, expected_keys);
          t.done();
        }
      });
    };
}, 'add() autoIncrement and out-of-line keys');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "test.obj.key",
        autoIncrement: true });

      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
      objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
      const actual_keys = [];
      const rq = db.transaction("store", "readonly", { durability: 'relaxed' })
        .objectStore("store")
        .openCursor();

      rq.onsuccess = t.step_func(function(e) {
        const cursor = e.target.result;

        if (cursor) {
          actual_keys.push(cursor.value.test.obj.key);
          cursor.continue();
        } else {
          assert_array_equals(actual_keys, expected_keys);
          t.done();
        }
      });
    };
}, 'Object store has autoIncrement:true and the key path is an object \
attribute');

async_test(t => {
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let rq;
      db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "key" });

      assert_throws_dom("DataError", function() {
        rq = objStore.add(record, 1);
      });

      assert_equals(rq, undefined);
      t.done();
    };
  }, 'Attempt to \'add()\' a record that does not meet the constraints of an \
  object store\'s inline key requirements');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;
      let rq;
      const objStore = db.createObjectStore("store");

      assert_throws_dom("DataError", function() {
        rq = objStore.add(record);
      });

      assert_equals(rq, undefined);
      t.done();
    };
}, 'Attempt to call \'add()\' without a key parameter when the object store \
uses out-of-line keys');

async_test(t => {
    const record = { key: { value: 1 }, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;

      let rq;
      const objStore = db.createObjectStore("store", { keyPath: "key" });

      assert_throws_dom("DataError", function() {
        rq = objStore.add(record);
      });

      assert_equals(rq, undefined);
      t.done();
    };
}, 'Attempt to \'add()\' a record where the record\'s key does not meet the \
constraints of a valid key');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;

      let rq;
      const objStore = db.createObjectStore("store", { keyPath: "key" });

      assert_throws_dom("DataError", function() {
        rq = objStore.add(record);
      });

      assert_equals(rq, undefined);
      t.done();
    };
}, 'Attempt to \'add()\' a record where the record\'s in-line key is not \
 defined');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;

      let rq;
      const objStore = db.createObjectStore("store");

      assert_throws_dom("DataError", function() {
        rq = objStore.add(record, { value: 1 });
      });

      assert_equals(rq, undefined);
      t.done();
    };
}, 'Attempt to \'add()\' a record where the out of line key provided does not \
meet the constraints of a valid key');

async_test(t => {
    const record = { key: 1, indexedProperty: { property: "data" } };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      let db = e.target.result;

      let rq;
      const objStore = db.createObjectStore("store", { keyPath: "key" });

      objStore.createIndex("index", "indexedProperty");

      rq = objStore.add(record);

      assert_true(rq instanceof IDBRequest);
      rq.onsuccess = function() {
          t.done();
      }
    };
}, 'add() a record where a value being indexed does not meet the constraints \
of a valid key');

async_test(t => {
    let db;

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function (event) {
        db = event.target.result;
        db.createObjectStore("store", {keyPath: "pKey"});
    }

    open_rq.onsuccess = function (event) {
        const txn = db.transaction("store", "readonly",
         {durability: 'relaxed'});
        const ostore = txn.objectStore("store");
        t.step(function() {
            assert_throws_dom("ReadOnlyError", function() {
                ostore.add({pKey: "primaryKey_0"});
            });
        });
        t.done();
    }
}, 'If the transaction this IDBObjectStore belongs to has its mode set to \
readonly, throw ReadOnlyError');

async_test(t => {
    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function (event) {
        let db = event.target.result;
        const ostore = db.createObjectStore("store", {keyPath: "pKey"});
        db.deleteObjectStore("store");
        assert_throws_dom("InvalidStateError", function() {
            ostore.add({pKey: "primaryKey_0"});
        });
        t.done();
    };
}, 'If the object store has been deleted, the implementation must throw a \
DOMException of type InvalidStateError');
