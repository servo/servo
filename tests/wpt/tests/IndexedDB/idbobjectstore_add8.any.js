// META: global=window,worker
// META: title=IDBObjectStore.add() - object store has autoIncrement:true and the key path is an object attribute
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
    const objStore = db.createObjectStore("store", { keyPath: "test.obj.key", autoIncrement: true });

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
            actual_keys.push(cursor.value.test.obj.key);
            cursor.continue();
        }
        else {
            assert_array_equals(actual_keys, expected_keys);
            t.done();
        }
    });
};
