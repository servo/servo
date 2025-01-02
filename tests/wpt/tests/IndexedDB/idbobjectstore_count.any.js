// META: global=window,worker
// META: title=IDBObjectStore.count()
// META: script=resources/support.js

'use strict';

async_test(t => {
    let db;

    let open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store");

        for(let i = 0; i < 10; i++) {
            store.add({ data: "data" + i }, i);
        }
    }

    open_rq.onsuccess = function(e) {
        let rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                   .objectStore("store")
                   .count();

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, 10);
            t.done();
        });
    }
}, "Returns the number of records in the object store ");

async_test(t => {
    let db;

    let open_rq = createdb(t);

    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        let store = db.createObjectStore("store");

        for(let i = 0; i < 10; i++) {
            store.add({ data: "data" + i }, i);
        }
    }

    open_rq.onsuccess = function(e) {
        let rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                   .objectStore("store")
                   .count(IDBKeyRange.bound(5, 20));

        rq.onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, 5);
            t.done();
        });
    }
}, "Returns the number of records that have keys within the range ");

async_test(t => {
    let db

    createdb(t).onupgradeneeded = function(e) {
        db = e.target.result

        let store = db.createObjectStore("store", { keyPath: "k" })

        for (let i = 0; i < 5; i++)
            store.add({ k: "key_" + i });

        store.count("key_2").onsuccess = t.step_func(function(e) {
            assert_equals(e.target.result, 1, "count(key_2)")

            store.count("key_").onsuccess = t.step_func(function(e) {
                assert_equals(e.target.result, 0, "count(key_)")
                t.done()
            })
        })
    }

}, "Returns the number of records that have keys with the key");

async_test(t => {
    let db, ostore;

    let open_rq = createdb(t);
    open_rq.onupgradeneeded = function (event) {
        db = event.target.result;
        ostore = db.createObjectStore("store", {keyPath:"pKey"});
        db.deleteObjectStore("store");
        assert_throws_dom("InvalidStateError", function(){
            ostore.count();
        });
        t.done();
    }
}, "If the object store has been deleted, the implementation must throw a DOMException of type InvalidStateError ");
