// META: title=IndexedDB: Test IDBObjectStore.getAllKeys
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js

'use strict';

function createGetAllKeysRequest(t, storeName, connection, keyRange, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const req = store.getAllKeys(keyRange, maxCount);
  req.onerror = t.unreached_func('getAllKeys request should succeed');
  return req;
}

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, 'c');
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['c']);
    t.done();
  });
}, 'Single item get');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'generated', connection, 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_array_equals(data, [3]);
    t.done();
  });
}, 'Single item get (generated key)');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'empty', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAllKeys() on empty object store should return an empty ' +
      'array');
    t.done();
  });
}, 'getAllKeys on empty object store');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet);
    t.done();
  });
}, 'Get all values');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, undefined,
    10);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'abcdefghij'.split(''));
    t.done();
  });
}, 'Test maxCount');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'ghijklm'.split(''));
    t.done();
  });
}, 'Get bound range');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'), 3);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i']);
    t.done();
  });
}, 'Get bound range with maxCount');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', false, true));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i', 'j']);
    t.done();
  });
}, 'Get upper excluded');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', true, false));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['h', 'i', 'j', 'k']);
    t.done();
  });
}, 'Get lower excluded');

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'generated', connection,
    IDBKeyRange.bound(4, 15), 3);
  req.onsuccess = t.step_func(evt => {
    const data = evt.target.result;
    assert_true(Array.isArray(data));
    assert_array_equals(data, [4, 5, 6]);
    t.done();
  });
}, 'Get bound range (generated) with maxCount');

object_store_get_all_test((t, connection) => {
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

object_store_get_all_test((t, connection) => {
  const req = createGetAllKeysRequest(t, 'out-of-line', connection, undefined,
    0);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet);
    t.done();
  });
}, 'zero maxCount');

object_store_get_all_test((test, connection) => {
  const request = createGetAllKeysRequest(
      test, 'out-of-line', connection, /*query=*/ undefined,
      /*count=*/ 4294967295);
  request.onsuccess = test.step_func(event => {
    assert_array_equals(event.target.result, alphabet);
    test.done();
  });
}, 'Max value count');

object_store_get_all_test((test, connection) => {
  const request = createGetAllKeysRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.upperBound('0'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAllKeys() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where  first key < upperBound');

object_store_get_all_test((test, connection) => {
  const request = createGetAllKeysRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.lowerBound('zz'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAllKeys() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where lowerBound < last key');
