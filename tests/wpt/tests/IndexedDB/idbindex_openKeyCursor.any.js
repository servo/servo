// META: global=window,worker
// META: title=IDBIndex.openKeyCursor()
// META: script=resources/support.js

'use strict';

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store", { keyPath: "key" });
        let index = store.createIndex("index", "indexedProperty");

        store.add({ key: 1, indexedProperty: "data" });

        assert_throws_dom("DataError", function(){
            index.openKeyCursor(NaN);
        });
        t.done();
    }
}, "Throw DataError when using a invalid key");

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store", { keyPath: "key" });
        let index = store.createIndex("index", "indexedProperty");

        store.add({ key: 1, indexedProperty: "data" });
        store.deleteIndex("index");

        assert_throws_dom("InvalidStateError", function(){
            index.openKeyCursor();
        });
        t.done();
    }
}, "Throw InvalidStateError when the index is deleted");

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store", { keyPath: "key" });
        let index = store.createIndex("index", "indexedProperty");
        store.add({ key: 1, indexedProperty: "data" });
    }
    open_rq.onsuccess = function(e) {
        db = e.target.result;
        let tx = db.transaction('store', 'readonly');
        let index = tx.objectStore('store').index('index');
        tx.abort();

        assert_throws_dom("TransactionInactiveError", function(){
            index.openKeyCursor();
        });
        t.done();
    }
}, "Throw TransactionInactiveError on aborted transaction");

async_test(t => {
    let db;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store", { keyPath: "key" });
        let index = store.createIndex("index", "indexedProperty");
        store.add({ key: 1, indexedProperty: "data" });

        e.target.transaction.abort();

        assert_throws_dom("InvalidStateError", function(){
            index.openKeyCursor();
        });
        t.done();
    }
}, "Throw InvalidStateError on index deleted by aborted upgrade");
