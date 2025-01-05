// META: global=window,worker
// META: title=IDBObjectStore.delete()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

async_test(t => {
    let db;
    const record = { key: 1, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;

        const objStore = db.createObjectStore("test", { keyPath: "key" });
        objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
        const delete_rq = db.transaction("test", "readwrite")
                          .objectStore("test")
                          .delete(record.key);

        delete_rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);

            e.target.transaction.oncomplete = t.step_func(VerifyRecordRemoved);
        });
    };

    function VerifyRecordRemoved() {
        const rq = db.transaction("test", "readonly")
                   .objectStore("test")
                   .get(record.key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
            t.done();
        });
    }
}, 'delete() removes record (inline keys)');

async_test(t => {
    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        const db = e.target.result;

        const delete_rq = db.createObjectStore("test")
                          .delete(1);

        delete_rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
            t.done();
        });
    };
}, 'delete() key doesn\'t match any records');

async_test(t => {
    let db;
    const record = { test: { obj: { key: 1 } }, property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;

        const objStore = db.createObjectStore("test",
        { keyPath: "test.obj.key" });
        objStore.add(record);
    };

    open_rq.onsuccess = function(e) {
        const delete_rq = db.transaction("test", "readwrite")
                          .objectStore("test")
                          .delete(record.test.obj.key);

        delete_rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);

            e.target.transaction.oncomplete = t.step_func(VerifyRecordRemoved);
        });
    };

    function VerifyRecordRemoved() {
        const rq = db.transaction("test", "readonly")
                   .objectStore("test")
                   .get(record.test.obj.key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
            t.done();
        });
    }
}, 'Object store\'s key path is an object attribute');

async_test(t => {
    let db;
    const key = 1;
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;

        const objStore = db.createObjectStore("test");
        objStore.add(record, key);
    };

    open_rq.onsuccess = function(e) {
        const delete_rq = db.transaction("test", "readwrite")
                          .objectStore("test")
                          .delete(key);

        delete_rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);

            e.target.transaction.oncomplete = t.step_func(VerifyRecordRemoved);
        });
    };

    function VerifyRecordRemoved() {
        const rq = db.transaction("test", "readonly")
                   .objectStore("test")
                   .get(key);

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
            t.done();
        });
    }
}, 'delete() removes record (out-of-line keys)');

async_test(t => {
    let db;
    const open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const os = db.createObjectStore("store");

        for(let i = 0; i < 10; i++)
            os.add("data" + i, i);
    };

    open_rq.onsuccess = function (e) {
        const os = db.transaction("store", "readwrite")
                   .objectStore("store");

        os.delete(IDBKeyRange.bound(3, 6));
        os.count().onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, 6, "Count after deleting \
            3-6 from 10");
            t.done();
        });
    };
}, 'delete() removes all of the records in the range');

async_test(function(t) {
    let db;
    const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(event) {
        db = event.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "pKey" });
        for (let  i = 0; i < records.length; i++) {
            objStore.add(records[i]);
        }
    };

    open_rq.onsuccess = function(event) {
        const txn = db.transaction("store", "readonly");
        const ostore = txn.objectStore("store");
        t.step(function() {
            assert_throws_dom("ReadOnlyError", function() {
                ostore.delete("primaryKey_0");
            });
        });
        t.done();
    };
}, 'If the transaction this IDBObjectStore belongs to has its mode set to \
readonly, throw ReadOnlyError');

async_test(t => {
    let ostore;
    const records = [{ pKey: "primaryKey_0" }, { pKey: "primaryKey_1" }];

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(event) {
        const db = event.target.result;
        ostore = db.createObjectStore("store", { keyPath: "pKey" });
        db.deleteObjectStore("store");
        assert_throws_dom("InvalidStateError", function() {
            ostore.delete("primaryKey_0");
        });
        t.done();
    };
}, 'If the object store has been deleted, the implementation must throw a \
DOMException of type InvalidStateError');
