// META: title=IndexedDB: Test IDBObjectStore.getAll
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

function createGetAllRequest(t, storeName, connection, keyRange, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const req = store.getAll(keyRange, maxCount);
  req.onerror = t.unreached_func('getAll request should succeed');
  return req;
}

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, 'c');
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['value-c']);
    t.done();
  });
}, 'Single item get');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'generated', connection, 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_equals(data.length, 1);
    assert_equals(data[0].id, 3);
    assert_equals(data[0].ch, 'c');
    t.done();
  });
}, 'Single item get (generated key)');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'empty', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAll() on empty object store should return an empty array');
    t.done();
  });
}, 'getAll on empty object store');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet.map(c => `value-${c}`));
    t.done();
  });
}, 'Get all values');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, undefined,
    10);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'abcdefghij'.split('').map(c => `value-${c}`));
    t.done();
  });
}, 'Test maxCount');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'ghijklm'.split('').map(c => `value-${c}`));
    t.done();
  });
}, 'Get bound range');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'), 3);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get bound range with maxCount');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', false, true));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i', 'j'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get upper excluded');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', true, false));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['h', 'i', 'j', 'k'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get lower excluded');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'generated', connection,
    IDBKeyRange.bound(4, 15), 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_array_equals(data.map(e => e.ch), ['d', 'e', 'f']);
    assert_array_equals(data.map(e => e.id), [4, 5, 6]);
    t.done();
  });
}, 'Get bound range (generated) with maxCount');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    "Doesn't exist");
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAll() using a nonexistent key should return an empty array');
    t.done();
  });
  req.onerror = t.unreached_func('getAll request should succeed');
}, 'Non existent key');

getall_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, undefined, 0);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet.map(c => `value-${c}`));
    t.done();
  });
}, 'zero maxCount');

getall_test((t, connection) => {
  const transaction = connection.transaction('out-of-line', 'readonly');
  const store = transaction.objectStore('out-of-line');
  const req = store.getAll();
  transaction.commit();
  transaction.oncomplete =
      t.unreached_func('transaction completed before request succeeded');

  req.onerror = t.unreached_func('getAll request should succeed');
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet.map(c => `value-${c}`));
    transaction.oncomplete = t.step_func_done();
  });
}, 'Get all values with transaction.commit()');
