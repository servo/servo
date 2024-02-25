// META: global=window,worker
// META: title=IDBObjectStore.add() - Attempt to add a record where the out of line key provided does not meet the constraints of a valid key
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let db;
const t = async_test();
const record = { property: "data" };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
    db = e.target.result;

    let rq;
    const objStore = db.createObjectStore("store");

    assert_throws_dom("DataError",
        function() { rq = objStore.add(record, { value: 1 }); });

    assert_equals(rq, undefined);
    t.done();
};
