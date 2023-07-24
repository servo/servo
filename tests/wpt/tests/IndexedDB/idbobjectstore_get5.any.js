// META: title=IDBObjectStore.get() - returns the record with the first key in the range
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

"use strict";

let db;
const t = async_test();
const open_rq = createdb(t);

open_rq.onupgradeneeded = event => {
  db = event.target.result;
  const os = db.createObjectStore("store");

  for (let i = 0; i < 10; i++) {
    os.add(`data${i}`, i);
  }
};

open_rq.onsuccess = event => {
  const rq = db.transaction("store", "readonly", {durability: 'relaxed'})
    .objectStore("store")
    .get(IDBKeyRange.bound(3, 6));

  rq.onsuccess = t.step_func(event => {
    assert_equals(event.target.result, "data3", "get(3-6)");
    t.done();
  });
};
