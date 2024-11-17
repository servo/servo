// META: title=IndexedDB: Test IDBObjectStore.getAll
// META: global=window,worker
// META: script=resources/nested-cloning-common.js
// META: script=resources/support.js
// META: script=resources/support-get-all.js
// META: script=resources/support-promises.js

'use strict';

function createGetAllRequest(t, storeName, connection, keyRange, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const req = store.getAll(keyRange, maxCount);
  req.onerror = t.unreached_func('getAll request should succeed');
  return req;
}

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, 'c');
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['value-c']);
    t.done();
  });
}, 'Single item get');

object_store_get_all_test((t, connection) => {
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

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'empty', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAll() on empty object store should return an empty array');
    t.done();
  });
}, 'getAll on empty object store');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet.map(c => `value-${c}`));
    t.done();
  });
}, 'Get all values');

object_store_get_all_test((test, connection) => {
  const request = createGetAllRequest(test, 'large-values', connection);
  request.onsuccess = test.step_func(event => {
    const actualResults = event.target.result;
    assert_true(Array.isArray(actualResults), 'The results must be an array');

    const expectedRecords = expectedObjectStoreRecords['large-values'];
    assert_equals(
        actualResults.length, expectedRecords.length,
        'The results array must contain the expected number of records');

    // Verify each large value.
    for (let i = 0; i < expectedRecords.length; i++) {
      assert_large_array_equals(
          actualResults[i], expectedRecords[i].value,
          'The record must have the expected value');
    }
    test.done();
  });
}, 'Get all with large values');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, undefined,
    10);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'abcdefghij'.split('').map(c => `value-${c}`));
    t.done();
  });
}, 'Test maxCount');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, 'ghijklm'.split('').map(c => `value-${c}`));
    t.done();
  });
}, 'Get bound range');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'm'), 3);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get bound range with maxCount');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', false, true));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['g', 'h', 'i', 'j'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get upper excluded');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    IDBKeyRange.bound('g', 'k', true, false));
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, ['h', 'i', 'j', 'k'].map(c => `value-${c}`));
    t.done();
  });
}, 'Get lower excluded');

object_store_get_all_test((t, connection) => {
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

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection,
    "Doesn't exist");
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, [],
      'getAll() using a nonexistent key should return an empty array');
    t.done();
  });
  req.onerror = t.unreached_func('getAll request should succeed');
}, 'Non existent key');

object_store_get_all_test((t, connection) => {
  const req = createGetAllRequest(t, 'out-of-line', connection, undefined, 0);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(evt.target.result, alphabet.map(c => `value-${c}`));
    t.done();
  });
}, 'zero maxCount');

object_store_get_all_test((test, connection) => {
  const request = createGetAllRequest(
      test, 'out-of-line', connection, /*query=*/ undefined,
      /*count=*/ 4294967295);
  request.onsuccess = test.step_func(event => {
    assert_array_equals(event.target.result, alphabet.map(c => `value-${c}`));
    test.done();
  });
}, 'Max value count');

object_store_get_all_test((test, connection) => {
  const request = createGetAllRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.upperBound('0'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAll() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where  first key < upperBound');

object_store_get_all_test((test, connection) => {
  const request = createGetAllRequest(
      test, /*storeName=*/ 'out-of-line', connection,
      IDBKeyRange.lowerBound('zz'));
  request.onsuccess = test.step_func((event) => {
    assert_array_equals(
        event.target.result, /*expectedResults=*/[],
        'getAll() with an empty query range must return an empty array');
    test.done();
  });
}, 'Query with empty range where lowerBound < last key');

object_store_get_all_test((t, connection) => {
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
