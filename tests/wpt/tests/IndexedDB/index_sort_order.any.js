// META: title= IDBIndex Key sort order
// META: global=window,worker
// META: script=resources/support.js

'use strict';

async_test(t => {
  let db;
  const d = new Date();
  const records = [{foo: d}, {foo: 'test'}, {foo: 1}, {foo: 2.55}];
  const expectedKeyOrder = [1, 2.55, d.valueOf(), 'test'];

  const open_rq = createdb(t);
  open_rq.onupgradeneeded = t.step_func((e) => {
    db = e.target.result;
    const objStore = db.createObjectStore('store', {autoIncrement: true});
    objStore.createIndex('index', 'foo');

    for (let i = 0; i < records.length; i++) {
      objStore.add(records[i]);
    }
  });

  open_rq.onsuccess = t.step_func((e) => {
    let actual_keys = [];
    const rq = db.transaction('store', 'readonly')
                   .objectStore('store')
                   .index('index')
                   .openCursor();

    rq.onsuccess = t.step_func((e) => {
      const cursor = e.target.result;

      if (cursor) {
        actual_keys.push(cursor.key.valueOf());
        cursor.continue();
      } else {
        assert_array_equals(actual_keys, expectedKeyOrder);
        t.done();
      }
    });
  });
}, 'Verify IDBIndex key sort order is \'number < Date < DOMString\'');
