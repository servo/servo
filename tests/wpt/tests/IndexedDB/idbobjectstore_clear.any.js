// META: global=window,worker
// META: title=IDBObjectStore.clear()
// META: script=resources/support.js

'use strict';

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let objStore = db.createObjectStore("store", { autoIncrement: true });

        objStore.add({ property: "data" });
        objStore.add({ something_different: "Yup, totally different" });
        objStore.add(1234);
        objStore.add([1, 2, 1234]);

        objStore.clear().onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
        });
    };


    open_rq.onsuccess = function(e) {
        let rq = db.transaction("store", "readonly")
                   .objectStore("store")
                   .openCursor();

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, null, 'cursor');
            t.done();
        });
    };
}, "Verify clear removes all records ");

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let objStore = db.createObjectStore("store", { autoIncrement: true });
        objStore.createIndex("index", "indexedProperty");

        objStore.add({ indexedProperty: "data" });
        objStore.add({ indexedProperty: "yo, man", something_different: "Yup, totally different" });
        objStore.add({ indexedProperty: 1234 });
        objStore.add({ indexedProperty: [1, 2, 1234] });
        objStore.add(1234);

        objStore.clear().onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, undefined);
        });
    };

    open_rq.onsuccess = function(e) {
        let rq = db.transaction("store", "readonly")
                   .objectStore("store")
                   .index("index")
                   .openCursor();

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, null, 'cursor');
            t.done();
        });
    };
}, "Clear removes all records from an index ");

async_test(t => {
    let db, records = [{ pKey: "primaryKey_0"}, { pKey: "primaryKey_1"}];

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function (event) {
        db = event.target.result;
        let objStore = db.createObjectStore("store", {keyPath:"pKey"});
        for (let i = 0; i < records.length; i++) {
            objStore.add(records[i]);
        }
    }

    open_rq.onsuccess = function (event) {
        let txn = db.transaction("store", "readonly");
        let ostore = txn.objectStore("store");
        t.step(function(){
            assert_throws_dom("ReadOnlyError", function(){
                ostore.clear();
            });
        });
        t.done();
    }
}, "If the transaction this IDBObjectStore belongs to has its mode set to readonly, throw ReadOnlyError ");

async_test(t => {
    let db, ostore;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function (event) {
        db = event.target.result;
        ostore = db.createObjectStore("store", {keyPath:"pKey"});
        db.deleteObjectStore("store");
        assert_throws_dom("InvalidStateError", function(){
            ostore.clear();
        });
        t.done();
    }
}, "If the object store has been deleted, the implementation must throw a DOMException of type InvalidStateError ");
