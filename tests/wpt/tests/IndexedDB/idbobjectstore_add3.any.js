// META: global=window,worker
// META: title=IDBObjectStore.add() - record with same key already exists
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

    const rq = objStore.add(record);
    rq.onsuccess = fail(t, "success on adding duplicate record");

    rq.onerror = t.step_func(function(e) {
        assert_equals(e.target.error.name, "ConstraintError");
        assert_equals(rq.error.name, "ConstraintError");
        assert_equals(e.type, "error");

        e.preventDefault();
        e.stopPropagation();
    });
};

// Defer done, giving rq.onsuccess a chance to run
open_rq.onsuccess = function(e) {
    t.done();
}
