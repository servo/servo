// META: title=IDBObjectStore.get() - throw TransactionInactiveError on aborted transaction
// META: script=resources/support.js
// @author YuichiNukiyama <https://github.com/YuichiNukiyama>

"use strict";

let db;
const t = async_test();

const open_rq = createdb(t);
open_rq.onupgradeneeded = event => {
  db = event.target.result;
  db.createObjectStore("store", { keyPath: "key" });
};

open_rq.onsuccess = event => {
  const store = db.transaction("store", "readonly", {durability: 'relaxed'})
    .objectStore("store");
  store.transaction.abort();
  assert_throws_dom("TransactionInactiveError", function () {
    store.get(1);
  }, "throw TransactionInactiveError on aborted transaction.");
  t.done();
};
