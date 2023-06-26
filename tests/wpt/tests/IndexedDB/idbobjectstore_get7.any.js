// META: title=IDBObjectStore.get() - throw DataError when using invalid key
// META: script=resources/support.js
// @author YuichiNukiyama <https://github.com/YuichiNukiyama>

"use strict";

let db;
const t = async_test();

const open_rq = createdb(t);
open_rq.onupgradeneeded = event => {
  db = event.target.result;
  db.createObjectStore("store", { keyPath: "key" });
}

open_rq.onsuccess = () => {
  const store = db.transaction("store", "readonly", {durability: 'relaxed'})
    .objectStore("store");
  assert_throws_dom("DataError", () => {
    store.get(null)
  }, "throw DataError when using invalid key.");
  t.done();
}
