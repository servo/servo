// META: global=window,worker
// META: title=IDBObjectStore.add() - add with an inline key
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { key: 1, property: "data" };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    const objStore = db.createObjectStore("store", { keyPath: "key" });

    objStore.add(record);
};

open_rq.onsuccess = function(e) {
    const rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                .objectStore("store")
                .get(record.key);

    rq.onsuccess = t.step_func(function(e) {
        assert_equals(e.target.result.property, record.property);
        assert_equals(e.target.result.key, record.key);
        t.done();
    });
};
