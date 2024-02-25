// META: global=window,worker
// META: title=IDBObjectStore.add() - add where an index has unique:true specified
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { key: 1, property: "data" };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;
    const objStore = db.createObjectStore("store", { autoIncrement: true });
    objStore.createIndex("i1", "property", { unique: true });
    objStore.add(record);

    const rq = objStore.add(record);
    rq.onsuccess = fail(t, "success on adding duplicate indexed record");

    rq.onerror = t.step_func(function(e) {
        assert_equals(rq.error.name, "ConstraintError");
        assert_equals(e.target.error.name, "ConstraintError");
        assert_equals(e.type, "error");

        e.preventDefault();
        e.stopPropagation();
    });
};

// Defer done, giving a spurious rq.onsuccess a chance to run
open_rq.onsuccess = function(e) {
    t.done();
}
