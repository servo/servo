// META: title=Batch Get All
// META: script=support.js

'use strict';

const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');

function batchgetall_test(storeName, func, name) {
  indexeddb_test((t, connection, tx) => {
    var store;
    switch (storeName) {
      case 'generated':
        store = connection.createObjectStore(
            'generated', {autoIncrement: true, keyPath: 'id'});
        alphabet.forEach(letter => {
          store.put({ch: letter});
        });
        break;
      case 'out-of-line':
        store = connection.createObjectStore('out-of-line', null);
        alphabet.forEach(letter => {
          store.put(`value-${letter}`, letter);
        });
        break;
      case 'empty':
        store = connection.createObjectStore('empty', null);
        break;
      default:
        t.fail(`Unsupported storeName: ${storeName}`);
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
  const req = createBatchGetAllRequest(t, 'out-of-line', connection, ['c']);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [['value-c']];
    assertTwoDArrayEquals(result, expected)
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
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, ['c', 'dd', 'e', 'ff']);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [['value-c'], [], ['value-e'], []];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'batchGetAll with non-existing values');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound('a', 'z')], 5);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [['value-a', 'value-b', 'value-c', 'value-d', 'value-e']];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range with maxCount');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound('a', 'e')]);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [['value-a', 'value-b', 'value-c', 'value-d', 'value-e']];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range');

batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(t, 'out-of-line', connection, [
    IDBKeyRange.bound('g', 'k', false, true),
    IDBKeyRange.bound('g', 'k', true, false)
  ]);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [
      ['value-g', 'value-h', 'value-i', 'value-j'],
      ['value-h', 'value-i', 'value-j', 'value-k']
    ];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get upper/lower excluded');

batchgetall_test('generated', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'generated', connection,
      [IDBKeyRange.bound(4, 15), IDBKeyRange.bound(5, 15)], 3);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [
      [{ch: 'd', id: 4}, {ch: 'e', id: 5}, {ch: 'f', id: 6}],
      [{ch: 'e', id: 5}, {ch: 'f', id: 6}, {ch: 'g', id: 7}]
    ];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range (generated) with maxCount');


batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound('a', 'e')], 0);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [['value-a', 'value-b', 'value-c', 'value-d', 'value-e']];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'zero maxCount');