// META: global=window,worker
// META: title=IDBObjectStore.add() - add with an out-of-line key
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const key = 1;
const record = { property: "data" };

var open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    const objStore = db.createObjectStore("store");

    objStore.add(record, key);
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
