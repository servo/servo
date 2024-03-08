// META: global=window,worker
// META: title=IndexedDB: Test IDBIndex.getAll
// META: script=resources/support.js

'use_strict';

const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');
const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

function getall_test(func, name) {
  indexeddb_test(
    function(t, connection, tx) {
      let store = connection.createObjectStore('generated',
            {autoIncrement: true, keyPath: 'id'});
      let index = store.createIndex('test_idx', 'upper');
      alphabet.forEach(function(letter) {
        store.put({ch: letter, upper: letter.toUpperCase()});
      });

      store = connection.createObjectStore('out-of-line', null);
      index = store.createIndex('test_idx', 'upper');
      alphabet.forEach(function(letter) {
        store.put({ch: letter, upper: letter.toUpperCase()}, letter);
      });

      store = connection.createObjectStore('out-of-line-not-unique', null);
      index = store.createIndex('test_idx', 'half');
      alphabet.forEach(function(letter) {
        if (letter <= 'm')
          store.put({ch: letter, half: 'first'}, letter);
        else
          store.put({ch: letter, half: 'second'}, letter);
      });

      store = connection.createObjectStore('out-of-line-multi', null);
      index = store.createIndex('test_idx', 'attribs', {multiEntry: true});
      alphabet.forEach(function(letter) {
        attrs = [];
        if (['a', 'e', 'i', 'o', 'u'].indexOf(letter) != -1)
          attrs.push('vowel');
        else
          attrs.push('consonant');
        if (letter == 'a')
          attrs.push('first');
        if (letter == 'z')
          attrs.push('last');
        store.put({ch: letter, attribs: attrs}, letter);
      });

      store = connection.createObjectStore('empty', null);
      index = store.createIndex('test_idx', 'upper');
    },
    func,
    name
  );
}

function createGetAllRequest(t, storeName, connection, range, maxCount) {
    const transaction = connection.transaction(storeName, 'readonly');
    const store = transaction.objectStore(storeName);
    const index = store.index('test_idx');
    const req = index.getAll(range, maxCount);
    req.onerror = t.unreached_func('getAll request should succeed');
    return req;
}

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection, 'C');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), ['c']);
          assert_array_equals(data.map(function(e) { return e.upper; }), ['C']);
          t.done();
      });
    }, 'Single item get');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'empty', connection);
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAll() on empty object store should return an empty array');
          t.done();
      });
    }, 'Empty object store');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), alphabet);
          assert_array_equals(data.map(function(e) { return e.upper; }), ALPHABET);
          t.done();
      });
    }, 'Get all keys');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection, undefined,
                                    10);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'abcdefghij'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'ABCDEFGHIJ'.split(''));
          t.done();
      });
    }, 'maxCount=10');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghijklm'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHIJKLM'.split(''));
          t.done();
      });
    }, 'Get bound range');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
                                    IDBKeyRange.bound('G', 'M'), 3);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghi'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHI'.split(''));
          t.done();
      });
    }, 'Get bound range with maxCount');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', false, true));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'ghij'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'GHIJ'.split(''));
          t.done();
      });
    }, 'Get upper excluded');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          IDBKeyRange.bound('G', 'K', true, false));
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'hijk'.split(''));
          assert_array_equals(data.map(function(e) { return e.upper; }), 'HIJK'.split(''));
          t.done();
      });
    }, 'Get lower excluded');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'generated',
          connection, IDBKeyRange.bound(4, 15), 3);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_true(Array.isArray(data));
          assert_equals(data.length, 0);
          t.done();
      });
    }, 'Get bound range (generated) with maxCount');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line',
          connection, "Doesn't exist");
      req.onsuccess = t.step_func(function(evt) {
          assert_array_equals(evt.target.result, [],
              'getAll() using a nonexistent key should return an empty array');
          t.done();
      req.onerror = t.unreached_func('getAll request should succeed');
      });
    }, 'Non existent key');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line', connection,
          undefined, 0);
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), alphabet);
          assert_array_equals(data.map(function(e) { return e.upper; }), ALPHABET);
          t.done();
      });
    }, 'maxCount=0');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line-not-unique', connection,
                                    'first');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), 'abcdefghijklm'.split(''));
          assert_true(data.every(function(e) { return e.half === 'first'; }));
          t.done();
      });
    }, 'Retrieve multiEntry key');

getall_test(function(t, connection) {
      const req = createGetAllRequest(t, 'out-of-line-multi', connection,
                                    'vowel');
      req.onsuccess = t.step_func(function(evt) {
          const data = evt.target.result;
          assert_class_string(data, 'Array', 'result should be an array');
          assert_array_equals(data.map(function(e) { return e.ch; }), ['a', 'e', 'i', 'o', 'u']);
          assert_array_equals(data[0].attribs, ['vowel', 'first']);
          assert_true(data.every(function(e) { return e.attribs[0] === 'vowel'; }));
          t.done();
      });
    }, 'Retrieve one key multiple values');
