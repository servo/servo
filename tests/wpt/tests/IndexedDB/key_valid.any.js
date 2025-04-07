// META: global=window,worker
// META: title=Valid key
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#key-construct

'use strict';

const valid_key = (desc, key) => {
  async_test(t => {
    let db;
    const open_rq = createdb(t);
    open_rq.onupgradeneeded = t.step_func(e => {
      db = e.target.result;
      const store = db.createObjectStore('store');
      assert_true(store.add('value', key) instanceof IDBRequest);

      const store2 = db.createObjectStore('store2', {
        keyPath: ['x', 'keypath'],
      });
      assert_true(store2.add({x: 'v', keypath: key}) instanceof IDBRequest);
    });

    open_rq.onsuccess = t.step_func(e => {
      const rq =
          db.transaction('store', 'readonly').objectStore('store').get(key);
      rq.onsuccess = t.step_func(e => {
        assert_equals(e.target.result, 'value');
        const rq2 =
            db.transaction('store2', 'readonly').objectStore('store2').get([
              'v', key
            ]);
        rq2.onsuccess = t.step_func(e => {
          assert_equals(e.target.result.x, 'v');
          assert_key_equals(e.target.result.keypath, key);
          t.done();
        });
      });
    });
  }, 'Valid key - ' + desc);
};

// Date
valid_key('new Date()', new Date());
valid_key('new Date(0)', new Date(0));

// Array
valid_key('[]', []);
valid_key('new Array()', new Array());

valid_key('["undefined"]', ['undefined']);

// Float
valid_key('Infinity', Infinity);
valid_key('-Infinity', -Infinity);
valid_key('0', 0);
valid_key('1.5', 1.5);
valid_key('3e38', 3e38);
valid_key('3e-38', 3e38);

// String
valid_key('"foo"', 'foo');
valid_key('"\\n"', '\n');
valid_key('""', '');
valid_key('"\\""', '"');
valid_key('"\\u1234"', '\u1234');
valid_key('"\\u0000"', '\u0000');
valid_key('"NaN"', 'NaN');
