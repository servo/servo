// META: global=window,worker
// META: title=IDBObjectStore.add() - If the transaction this IDBObjectStore belongs to has its mode set to readonly, throw ReadOnlyError
// META: script=resources/support.js
// @author Intel <http://www.intel.com>

'use_strict';

let db;
const t = async_test();

const open_rq = createdb(t);
open_rq.onupgradeneeded = function (event) {
    db = event.target.result;
    db.createObjectStore("store", {keyPath:"pKey"});
}

open_rq.onsuccess = function (event) {
    const txn = db.transaction("store", "readonly", {durability: 'relaxed'});
    const ostore = txn.objectStore("store");
    t.step(function(){
        assert_throws_dom("ReadOnlyError", function(){
            ostore.add({ pKey: "primaryKey_0"});
        });
    });
    t.done();
}
