// META: global=window,worker
// META: title=IDBObjectStore.add() - Attempt to add a record that does not meet the constraints of an object store's inline key requirements
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

'use_strict';

let t = async_test();
const record = { key: 1, property: "data" };

const open_rq = createdb(t);
open_rq.onupgradeneeded = function(e) {
        let rq;
        db = e.target.result;
        const objStore = db.createObjectStore("store", { keyPath: "key" });

    assert_throws_dom("DataError",
        function() { rq = objStore.add(record, 1); });

    assert_equals(rq, undefined);
    t.done();
};
