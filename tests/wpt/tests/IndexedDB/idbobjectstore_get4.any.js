// META: title=IDBObjectStore.get() - attempt to retrieve a record that doesn't exist
// META: script=resources/support.js
// @author Microsoft <https://www.microsoft.com>

"use strict";

let db;
const t = async_test();

const open_rq = createdb(t);
open_rq.onupgradeneeded = event => {
  db = event.target.result;
  const rq = db.createObjectStore("store", { keyPath: "key" })
    .get(1);
  rq.onsuccess = t.step_func(event => {
    assert_equals(event.target.results, undefined);
    t.done();
  });
};
