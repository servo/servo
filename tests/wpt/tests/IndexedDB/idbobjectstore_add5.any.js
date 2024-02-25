// META: global=window,worker
// META: title=IDBObjectStore.add() - object store's key path is an object attribute
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { test: { obj: { key: 1 } }, property: "data" };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    const objStore = db.createObjectStore("store", { keyPath: "test.obj.key" });
    objStore.add(record);
};

open_rq.onsuccess = function(e) {
    const rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                .objectStore("store")
                .get(record.test.obj.key);

    rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.property, record.property);

        t.done();
    });
};
