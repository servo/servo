// META: global=window,worker
// META: title=IDBObjectStore.put()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>
// @author Intel <http://www.intel.com>

'use strict';

async_test(t => {
    let db;
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        objStore.put(record);
    };

    open_rq.onsuccess = function(e) {
        const rq = db.transaction("store", "readonly",
         { durability: 'relaxed' })
                   .objectStore("store")
                   .get(record.key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result.property, record.property);
            assert_equals(e.target.result.key, record.key);
            t.done();
        });
    };
}, 'put() with an inline key');

async_test(t => {
    let db;
    const key = 1;
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store");

        objStore.put(record, key);
    };

    open_rq.onsuccess = function(e) {
        const rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                   .objectStore("store")
                   .get(key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result.property, record.property);

            t.done();
        });
    };
},'put() with an out-of-line key');

async_test(t => {
    let db;
    let success_event;
    const record = { key: 1, property: "data" };
    const record_put = { key: 1, property: "changed", more: ["stuff", 2] };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "key" });
        objStore.put(record);

        const rq = objStore.put(record_put);
        rq.onerror = fail(t, "error on put");

        rq.onsuccess = t.step_func(function(e) {
            success_event = true;
        });
    };

    open_rq.onsuccess = function(e) {
        assert_true(success_event);

        const rq = db.transaction("store", "readonly",
         { durability: 'relaxed' })
                   .objectStore("store")
                   .get(1);

        rq.onsuccess = t.step_func(function(e) {
            const rec = e.target.result;

            assert_equals(rec.key, record_put.key);
            assert_equals(rec.property, record_put.property);
            assert_array_equals(rec.more, record_put.more);

            t.done();
        });
    };
}, 'put() record with key already exists');

async_test(t => {
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        let db = e.target.result;
        const objStore = db.createObjectStore("store", {
             autoIncrement: true });
        objStore.createIndex("i1", "property", { unique: true });
        objStore.put(record);

        const rq = objStore.put(record);
        rq.onsuccess = fail(t, "success on putting duplicate indexed record");

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
}, 'put() where an index has unique:true specified');

async_test(t => {
    let db;
    const record = { test: { obj: { key: 1 } }, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store",
        { keyPath: "test.obj.key" });
        objStore.put(record);
    };

    open_rq.onsuccess = function(e) {
        const rq = db.transaction("store", "readonly",
        { durability: 'relaxed' })
                   .objectStore("store")
                   .get(record.test.obj.key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result.property, record.property);

            t.done();
        });
    };
}, 'Object store\'s key path is an object attribute');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
      db = e.target.result;
      const objStore = db.createObjectStore("store", { keyPath: "key",
      autoIncrement: true });

      objStore.put(record);
      objStore.put(record);
      objStore.put(record);
      objStore.put(record);
    };

    open_rq.onsuccess = function(e) {
      let actual_keys = [];
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
  }, 'autoIncrement and inline keys');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "key",
        autoIncrement: true });

        objStore.put(record);
        objStore.put(record);
        objStore.put(record);
        objStore.put(record);
    };

    open_rq.onsuccess = function(e) {
        const actual_keys = [];
        const rq = db.transaction("store", "readonly",
        { durability: 'relaxed' })
                   .objectStore("store")
                   .openCursor();

        rq.onsuccess = t.step_func(function(e) {
            const cursor = e.target.result;

            if(cursor) {
                actual_keys.push(cursor.value.key);
                cursor.continue();
            } else {
                assert_array_equals(actual_keys, expected_keys);
                t.done();
            }
        });
    };
}, 'autoIncrement and out-of-line keys');

async_test(t => {
    let db;
    const record = { property: "data" };
    const expected_keys = [1, 2, 3, 4];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("store",
        { keyPath: "test.obj.key", autoIncrement: true });

        objStore.put(record);
        objStore.put(record);
        objStore.put(record);
        objStore.put(record);
    };

    open_rq.onsuccess = function(e) {
        const actual_keys = [];
        const rq = db.transaction("store", "readonly",
        { durability: 'relaxed' })
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
        const db = e.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        assert_throws_dom("DataError", function() {
            rq = objStore.put(record, 1);
        });

        assert_equals(rq, undefined);
        t.done();
    };
}, 'Attempt to put() a record that does not meet the constraints of an object \
store\'s inline key requirements');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        let db = e.target.result;

        let rq;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        assert_throws_dom("DataError", function() {
            rq = objStore.put(record);
        });

        assert_equals(rq, undefined);
        t.done();
    };
}, 'Attempt to call put() without an key parameter when the object store uses \
out-of-line keys');

async_test(t => {
    const record = { key: { value: 1 }, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        let rq;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        assert_throws_dom("DataError", function() {
            rq = objStore.put(record);
        });

        assert_equals(rq, undefined);
        t.done();
    };
}, 'Attempt to put() a record where the record\'s key does not meet the \
constraints of a valid key');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        let rq;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        assert_throws_dom("DataError", function() {
            rq = objStore.put(record);
        });

        assert_equals(rq, undefined);
        t.done();
    };
}, 'Attempt to put() a record where the record\'s in-line key is not defined');

async_test(t => {
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        let rq;
        const objStore = db.createObjectStore("store");

        assert_throws_dom("DataError", function() {
            rq = objStore.put(record, { value: 1 });
        });

        assert_equals(rq, undefined);
        t.done();
    };
}, 'Attempt to put() a record where the out of line key provided does not \
meet the constraints of a valid key');

async_test(t => {
    const record = { key: 1, indexedProperty: { property: "data" } };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        let rq;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

        objStore.createIndex("index", "indexedProperty");

        rq = objStore.put(record);

        assert_true(rq instanceof IDBRequest);
        rq.onsuccess = function() {
            t.done();
        };
    };
}, 'put() a record where a value being indexed does not meet the constraints \
of a valid key');

async_test(t => {
    let db;
    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(event) {
        db = event.target.result;
        db.createObjectStore("store", { keyPath: "pKey" });
    };

    open_rq.onsuccess = function(event) {
        const txn = db.transaction("store", "readonly",
        { durability: 'relaxed' });
        const ostore = txn.objectStore("store");
        t.step(function() {
            assert_throws_dom("ReadOnlyError", function() {
                ostore.put({ pKey: "primaryKey_0" });
            });
        });
        t.done();
    };
}, 'If the transaction this IDBObjectStore belongs to has its mode set to \
readonly, throw ReadOnlyError');

async_test(t => {
    let ostore;
    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(event) {
        const db = event.target.result;
        ostore = db.createObjectStore("store", { keyPath: "pKey" });
        db.deleteObjectStore("store");
        assert_throws_dom("InvalidStateError", function() {
            ostore.put({ pKey: "primaryKey_0" });
        });
        t.done();
    };
}, 'If the object store has been deleted, the implementation must throw a \
DOMException of type InvalidStateError');
