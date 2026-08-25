// META: script=/IndexedDB/resources/support.js
"use strict";

async_test(t => {
  const openReq = createdb(t);

  openReq.onupgradeneeded = e => {
    const db = e.target.result;
    const store = db.createObjectStore("store", { keyPath: "key" });
    // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
    const sab = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;

    assert_throws_dom("DataCloneError", () => {
      store.put({ key: 1, property: sab });
    });
    t.done();
  };
}, "SharedArrayBuffer cloning via IndexedDB: basic case");

async_test(t => {
  const openReq = createdb(t);

  openReq.onupgradeneeded = e => {
    const db = e.target.result;
    const store = db.createObjectStore("store", { keyPath: "key" });
    // See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`
    const sab = new WebAssembly.Memory({ shared:true, initial:1, maximum:1 }).buffer;

    let getter1Called = false;
    let getter2Called = false;

    assert_throws_dom("DataCloneError", () => {
      store.put({ key: 1, property: [
        { get x() { getter1Called = true; return 5; } },
        sab,
        { get x() { getter2Called = true; return 5; } }
      ]});
    });

    assert_true(getter1Called, "The getter before the SAB must have been called");
    assert_false(getter2Called, "The getter after the SAB must not have been called");
    t.done();
  };
}, "SharedArrayBuffer cloning via the IndexedDB: is interleaved correctly");
