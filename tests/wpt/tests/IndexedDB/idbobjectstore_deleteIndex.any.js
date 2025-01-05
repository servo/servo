// META: global=window,worker
// META: title=IDBObjectStore.deleteIndex()
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

async_test(t => {
    let db;
    const key = 1;
    const record = { property: "data" };

    const open_rq = createdb(t);
    open_rq.onupgradeneeded = function(e) {
        db = e.target.result;
        const objStore = db.createObjectStore("test");
        objStore.createIndex("index", "indexedProperty");
    };

    open_rq.onsuccess = function(e) {
        db.close();
        const new_version = createdb(t, db.name, 2);
        new_version.onupgradeneeded = function(e) {
            db = e.target.result;
            const objStore = e.target.transaction.objectStore("test");
            objStore.deleteIndex("index");
        };

        new_version.onsuccess = function(e) {
            let index;
            const objStore = db.transaction("test", "readonly")
                               .objectStore("test");

            assert_throws_dom('NotFoundError', function()
            { index = objStore.index("index"); });
            assert_equals(index, undefined);
            db.close();
            t.done();
        };
    };
}, 'IDBObjectStore.deleteIndex() removes the index');
