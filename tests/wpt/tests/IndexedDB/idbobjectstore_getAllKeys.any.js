// META: title=IndexedDB: Test IDBObjectStore.getAllKeys
// META: script=resources/support.js

'use strict';

const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');

function getall_test(func, name) {
  indexeddb_test(
    (t, connection, tx) => {
      let store = connection.createObjectStore('generated',
        { autoIncrement: true, keyPath: 'id' });
      alphabet.forEach(letter => {
        store.put({ ch: letter });
      });

      store = connection.createObjectStore('out-of-line', null);
      alphabet.forEach(letter => {
        store.put(`value-${letter}`, letter);
      });

      store = connection.createObjectStore('empty', null);
    },
    func,
    name
  );
}

function createGetAllKeysRequest(t, storeName, connection, keyRange, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const req = store.getAllKeys(keyRange, maxCount);
  req.onerror = t.unreached_func('getAllKeys request should succeed');
  return req;
}

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, 'c');
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['c']);
    t.done();
  });
}, 'Single item get');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'generated', connection, 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_array_equals(data, [3]);
    t.done();
  });
}, 'Single item get (generated key)');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'empty', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAllKeys() on empty object store should return an empty ' +
      'array');
    t.done();
  });
}, 'getAllKeys on empty object store');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet);
    t.done();
  });
}, 'Get all values');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, undefined,
    10);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'abcdefghij'.split(''));
    t.done();
  });
}, 'Test maxCount');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'ghijklm'.split(''));
    t.done();
  });
}, 'Get bound range');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'), 3);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i']);
    t.done();
  });
}, 'Get bound range with maxCount');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', false, true));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i', 'j']);
    t.done();
  });
}, 'Get upper excluded');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', true, false));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['h', 'i', 'j', 'k']);
    t.done();
  });
}, 'Get lower excluded');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'generated', connection,
    IDBKeyRange.bound(4, 15), 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_array_equals(data, [4, 5, 6]);
    t.done();
  });
}, 'Get bound range (generated) with maxCount');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    "Doesn't exist");
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAllKeys() using a nonexistent key should return an ' +
      'empty array');
    t.done();
  });
  req.onerror = t.unreached_func('getAllKeys request should succeed');
}, 'Non existent key');

getall_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, undefined,
    0);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet);
    t.done();
  });
}, 'zero maxCount');
