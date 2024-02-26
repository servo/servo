// META: global=window,worker
// META: title=IDBObjectStore.add() - autoIncrement and out-of-line keys
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { property: "data" };
const expected_keys = [ 1, 2, 3, 4 ];

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
    const actual_keys = [],
        rq = db.transaction("store", "readonly", {durability: 'relaxed'})
                .objectStore("store")
                .openCursor();

    rq.onsuccess = t.step_func(function(e) {
        const cursor = e.target.result;

        if (cursor) {
            actual_keys.push(cursor.key);
            cursor.continue();
        }
        else {
            assert_array_equals(actual_keys, expected_keys);
            t.done();
        }
    });
};
