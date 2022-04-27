// META: title=Batch Get All (big value)
// META: script=support.js
// META: script=support-promises.js

'use strict';

// engines that have special code paths for large values.
const wrapThreshold = 128 * 1024;
const keys = Array.from({length: 10}, (item, index) => index);
const values =
    Array.from(keys, (item, index) => largeValue(wrapThreshold, index));

function batchgetall_test(storeName, func, name) {
  indexeddb_test((t, connection, tx) => {
    let store = connection.createObjectStore(storeName, null);
    for (let i = 0; i < keys.length; i++) {
      store.put(values[i], keys[i])
    }
  }, func, name);
}

function createBatchGetAllRequest(t, storeName, connection, ranges, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const req = store.batchGetAll(ranges, maxCount);
  req.onerror = t.unreached_func('batchGetAll request should succeed');
  return req;
}

function assertTwoDArrayEquals(result, expected) {
  assert_equals(JSON.stringify(result), JSON.stringify(expected));
}

batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(t, 'out-of-line', connection, [2]);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [[values[2]]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Single item get');


batchgetall_test('empty', (t, connection) => {
  const req = createBatchGetAllRequest(t, 'empty', connection);
  req.onsuccess = t.step_func(evt => {
    assert_array_equals(
        evt.target.result, [],
        'getAll() on empty object store should return an empty array');
    t.done();
  });
}, 'batchGetAll on empty object store');


batchgetall_test('out-of-line', (t, connection) => {
  const req =
      createBatchGetAllRequest(t, 'out-of-line', connection, [1, 'a', 4, 'z']);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [[values[1]], [], [values[4]], []];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'batchGetAll with non-existing values');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound(0, 10)], 5);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [[values[0], values[1], values[2], values[3], values[4]]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range with maxCount');



batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound(0, 4)]);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [[values[0], values[1], values[2], values[3], values[4]]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(t, 'out-of-line', connection, [
    IDBKeyRange.bound(0, 4, false, true), IDBKeyRange.bound(0, 4, true, false)
  ]);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [
      [values[0], values[1], values[2], values[3]],
      [values[1], values[2], values[3], values[4]]
    ];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get upper/lower excluded');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound(1, 4)], 0);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [[values[1], values[2], values[3], values[4]]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'zero maxCount');