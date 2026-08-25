// META: title= IndexedDB: Detached buffers supplied as binary keys
// META: global=window,worker
// META: script=resources/support.js

// Specs:
//   http://w3c.github.io/IndexedDB/#convert-a-value-to-a-key
//   https://webidl.spec.whatwg.org/#dfn-get-buffer-source-copy

"use strict";

indexeddb_test(
  (t, db) => { db.createObjectStore('store'); },
  (t, db) => {
    const tx = db.transaction('store', 'readwrite');
    const store = tx.objectStore('store');

    const array = createDetachedArrayBuffer();
    const buffer = array.buffer;
    assert_throws_dom("DataError", () => { store.put('', buffer); });
    assert_throws_dom("DataError", () => { store.put('', [buffer]); });
    t.done();
  },
  'Detached ArrayBuffers must throw DataError when used as a key'
);

indexeddb_test(
  (t, db) => { db.createObjectStore('store'); },
  (t, db) => {
    const tx = db.transaction('store', 'readwrite');
    const store = tx.objectStore('store');

    const array = createDetachedArrayBuffer();
    assert_throws_dom("DataError", () => { store.put('', array); });
    assert_throws_dom("DataError", () => { store.put('', [array]); });
    t.done();
  },
  'Detached TypedArrays must throw DataError when used as a key'
);
