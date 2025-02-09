// META: title=IndexedDB: recursive value
// META: global=window,worker
// META: script=resources/support.js

'use strict';

function recursive_value(desc, value) {
  let db;
  const t = async_test('Recursive value - ' + desc);

  createdb(t).onupgradeneeded = t.step_func((e) => {
    db = e.target.result;
    db.createObjectStore('store').add(value, 1);

    e.target.onsuccess = t.step_func((e) => {
      db.transaction('store', 'readonly')
          .objectStore('store')
          .get(1)
          .onsuccess = t.step_func((e) => {
        try {
          JSON.stringify(value);
          assert_unreached(
              'The test case is incorrect. It must provide a recursive value that JSON cannot stringify.');
        } catch (e) {
          if (e.name == 'TypeError') {
            try {
              JSON.stringify(e.target.result);
              assert_unreached(
                  'Expected a non-JSON-serializable value back, didn\'t get that.');
            } catch (e) {
              t.done();
              return;
            }
          } else
            throw e;
        }
      });
    });
  });
}

const recursive = [];
recursive.push(recursive);
recursive_value('array directly contains self', recursive);

const recursive2 = [];
recursive2.push([recursive2]);
recursive_value('array indirectly contains self', recursive2);

const recursive3 = [recursive];
recursive_value('array member contains self', recursive3);
