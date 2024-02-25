// META: global=window,worker
// META: title=IDBObjectStore.add() - If the object store has been deleted, the implementation must throw a DOMException of type InvalidStateError
// META: script=resources/support.js
// @author Intel <http://www.intel.com>

'use_strict';

let db;
let ostore;
const t = async_test();

const open_rq = createdb(t);
open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    ostore = db.createObjectStore("store", {keyPath:"pKey"});
    db.deleteObjectStore("store");
    assert_throws_dom("InvalidStateError", function(){
        ostore.add({ pKey: "primaryKey_0"});
    });
    t.done();
}
