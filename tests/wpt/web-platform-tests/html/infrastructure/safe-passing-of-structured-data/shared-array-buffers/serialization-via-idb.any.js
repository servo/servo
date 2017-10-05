// META: script=/IndexedDB/support.js
"use strict";

async_test(t => {
  const openReq = createdb(t);

  openReq.onupgradeneeded = e => {
    const db = e.target.result;
    const store = db.createObjectStore("store", { keyPath: "key" });

    assert_throws("DataCloneError", () => {
      store.put({ key: 1, property: new SharedArrayBuffer() });
    });
    t.done();
  };
}, "SharedArrayBuffer cloning via IndexedDB: basic case");

async_test(t => {
  const openReq = createdb(t);

  openReq.onupgradeneeded = e => {
    const db = e.target.result;
    const store = db.createObjectStore("store", { keyPath: "key" });

    let getter1Called = false;
    let getter2Called = false;

    assert_throws("DataCloneError", () => {
      store.put({ key: 1, property: [
        { get x() { getter1Called = true; return 5; } },
        new SharedArrayBuffer(),
        { get x() { getter2Called = true; return 5; } }
      ]});
    });

    assert_true(getter1Called, "The getter before the SAB must have been called");
    assert_false(getter2Called, "The getter after the SAB must not have been called");
    t.done();
  };
}, "SharedArrayBuffer cloning via the IndexedDB: is interleaved correctly");
