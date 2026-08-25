// META: title=IndexedDB
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  let trans;
  let store;
  let index;
  let request = createdb(t);

  request.onupgradeneeded = function(e) {
    db = e.target.result;
    store = db.createObjectStore('store');
    index = store.createIndex('index', 'value');
    store.put({value: 0}, 0);
    trans = request.transaction;
    trans.oncomplete = verifyOverloads;
  };

  function verifyOverloads() {
    trans = db.transaction('store', 'readonly');
    store = trans.objectStore('store');
    index = store.index('index');

    checkCursorDirection(store.openCursor(), 'next');
    checkCursorDirection(store.openCursor(0), 'next');
    checkCursorDirection(store.openCursor(0, 'next'), 'next');
    checkCursorDirection(store.openCursor(0, 'nextunique'), 'nextunique');
    checkCursorDirection(store.openCursor(0, 'prev'), 'prev');
    checkCursorDirection(store.openCursor(0, 'prevunique'), 'prevunique');

    checkCursorDirection(store.openCursor(IDBKeyRange.only(0)), 'next');
    checkCursorDirection(store.openCursor(IDBKeyRange.only(0), 'next'), 'next');
    checkCursorDirection(
        store.openCursor(IDBKeyRange.only(0), 'nextunique'), 'nextunique');
    checkCursorDirection(store.openCursor(IDBKeyRange.only(0), 'prev'), 'prev');
    checkCursorDirection(
        store.openCursor(IDBKeyRange.only(0), 'prevunique'), 'prevunique');

    checkCursorDirection(index.openCursor(), 'next');
    checkCursorDirection(index.openCursor(0), 'next');
    checkCursorDirection(index.openCursor(0, 'next'), 'next');
    checkCursorDirection(index.openCursor(0, 'nextunique'), 'nextunique');
    checkCursorDirection(index.openCursor(0, 'prev'), 'prev');
    checkCursorDirection(index.openCursor(0, 'prevunique'), 'prevunique');

    checkCursorDirection(index.openCursor(IDBKeyRange.only(0)), 'next');
    checkCursorDirection(index.openCursor(IDBKeyRange.only(0), 'next'), 'next');
    checkCursorDirection(
        index.openCursor(IDBKeyRange.only(0), 'nextunique'), 'nextunique');
    checkCursorDirection(index.openCursor(IDBKeyRange.only(0), 'prev'), 'prev');
    checkCursorDirection(
        index.openCursor(IDBKeyRange.only(0), 'prevunique'), 'prevunique');

    checkCursorDirection(index.openKeyCursor(), 'next');
    checkCursorDirection(index.openKeyCursor(0), 'next');
    checkCursorDirection(index.openKeyCursor(0, 'next'), 'next');
    checkCursorDirection(index.openKeyCursor(0, 'nextunique'), 'nextunique');
    checkCursorDirection(index.openKeyCursor(0, 'prev'), 'prev');
    checkCursorDirection(index.openKeyCursor(0, 'prevunique'), 'prevunique');

    checkCursorDirection(index.openKeyCursor(IDBKeyRange.only(0)), 'next');
    checkCursorDirection(
        index.openKeyCursor(IDBKeyRange.only(0), 'next'), 'next');
    checkCursorDirection(
        index.openKeyCursor(IDBKeyRange.only(0), 'nextunique'), 'nextunique');
    checkCursorDirection(
        index.openKeyCursor(IDBKeyRange.only(0), 'prev'), 'prev');
    checkCursorDirection(
        index.openKeyCursor(IDBKeyRange.only(0), 'prevunique'), 'prevunique');

    t.done();
  }

  function checkCursorDirection(request, direction) {
    request.onsuccess = function(event) {
      assert_not_equals(
          event.target.result, null, 'Check the result is not null')
      assert_equals(
          event.target.result.direction, direction,
          'Check the result direction');
    };
  }
}, 'Validate the overloads of IDBObjectStore.openCursor(), IDBIndex.openCursor() and IDBIndex.openKeyCursor()');
