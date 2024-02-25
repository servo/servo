// META: global=window,worker
// META: title=IDBObjectStore.add() - Add a record where a value being indexed does not meet the constraints of a valid key
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { key: 1, indexedProperty: { property: "data" } };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;

    let rq;
    const objStore = db.createObjectStore("store", { keyPath: "key" });

    objStore.createIndex("index", "indexedProperty");

    rq = objStore.add(record);

    assert_true(rq instanceof IDBRequest);
    rq.onsuccess = function() {
        t.done();
    }
};
