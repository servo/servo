// META: global=window,worker
// META: title=IDBCursor direction - index
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#cursor-iteration-operation

'use strict';

let records = ['Alice', 'Bob', 'Bob', 'Greg'];
let cases = [
  {dir: 'next', expect: ['Alice:0', 'Bob:1', 'Bob:2', 'Greg:3']},
  {dir: 'prev', expect: ['Greg:3', 'Bob:2', 'Bob:1', 'Alice:0']},
  {dir: 'nextunique', expect: ['Alice:0', 'Bob:1', 'Greg:3']},
  {dir: 'prevunique', expect: ['Greg:3', 'Bob:1', 'Alice:0']},
];

cases.forEach(function(testcase) {
  let dir = testcase.dir;
  let expect = testcase.expect;
  indexeddb_test(
      function(t, db, tx) {
        let objStore = db.createObjectStore('test');
        objStore.createIndex('idx', 'name');

        for (let i = 0; i < records.length; i++)
          objStore.add({name: records[i]}, i);
      },
      function(t, db) {
        let count = 0;
        let rq = db.transaction('test', 'readonly')
                     .objectStore('test')
                     .index('idx')
                     .openCursor(undefined, dir);
        rq.onsuccess = t.step_func(function(e) {
          let cursor = e.target.result;
          if (!cursor) {
            assert_equals(count, expect.length, 'cursor runs');
            t.done();
          }
          assert_equals(
              cursor.value.name + ':' + cursor.primaryKey, expect[count],
              'cursor.value');
          count++;
          cursor.continue();
        });
        rq.onerror = t.step_func(function(e) {
          e.preventDefault();
          e.stopPropagation();
          assert_unreached('rq.onerror - ' + e.message);
        });
      },
      'IDBCursor direction - index - ' + dir);
});
