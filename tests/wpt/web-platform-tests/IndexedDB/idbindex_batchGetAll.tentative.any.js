// META: title=Batch Get All Index
// META: script=support.js

'use strict';

const alphabet = 'abcdefghijklmnopqrstuvwxyz'.split('');
const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

function batchgetall_test(storeName, func, name) {
  indexeddb_test((t, connection, tx) => {
    var store;
    var index;
    switch (storeName) {
      case 'generated':
        store = connection.createObjectStore(
            'generated', {autoIncrement: true, keyPath: 'id'});
        index = store.createIndex('test_idx', 'upper');
        alphabet.forEach(letter => {
          store.put({ch: letter, upper: letter.toUpperCase()});
        });
        break;
      case 'out-of-line':
        store = connection.createObjectStore('out-of-line', null);
        index = store.createIndex('test_idx', 'upper');
        alphabet.forEach(letter => {
          store.put({ch: letter, upper: letter.toUpperCase()}, letter);
        });
        break;
      case 'out-of-line-not-unique':
        store = connection.createObjectStore('out-of-line-not-unique', null);
        index = store.createIndex('test_idx', 'half');
        alphabet.forEach(letter => {
          if (letter <= 'd')
            store.put({ch: letter, half: 'first'}, letter);
          else if (letter < 'x')
            store.put({ch: letter, half: 'second'}, letter);
          else
            store.put({ch: letter, half: 'third'}, letter);
        });
        break;
      case 'out-of-line-multi':
        store = connection.createObjectStore('out-of-line-multi', null);
        index = store.createIndex('test_idx', 'attribs', {multiEntry: true});
        alphabet.forEach(function(letter) {
          let attrs = [];
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
        break;
      case 'empty':
        store = connection.createObjectStore('empty', null);
        store.createIndex('test_idx', 'upper');
        break;
      default:
        t.fail(`Unsupported storeName: ${storeName}`);
    }
  }, func, name);
}

function createBatchGetAllRequest(t, storeName, connection, ranges, maxCount) {
  const transaction = connection.transaction(storeName, 'readonly');
  const store = transaction.objectStore(storeName);
  const index = store.index('test_idx');
  const req = index.batchGetAll(ranges, maxCount);
  req.onerror = t.unreached_func('batchGetsAll request should succeed');
  return req;
}

function assertTwoDArrayEquals(result, expected) {
  assert_equals(JSON.stringify(result), JSON.stringify(expected));
}

batchgetall_test('out-of-line', (t, connection) => {
  const req = createBatchGetAllRequest(t, 'out-of-line', connection, ['C']);
  req.onsuccess = t.step_func(evt => {
    let expected = [[{'ch': 'c', 'upper': 'C'}]];
    let result = evt.target.result;
    assert_class_string(result, 'Array', 'result should be an array');
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Single getAll');

batchgetall_test('out-of-line', (t, connection) => {
  const req =
      createBatchGetAllRequest(t, 'out-of-line', connection, ['C', 'D', 'E']);
  req.onsuccess = t.step_func(evt => {
    let result = evt.target.result;
    let expected = [
      [{'ch': 'c', 'upper': 'C'}], [{'ch': 'd', 'upper': 'D'}],
      [{'ch': 'e', 'upper': 'E'}]
    ];
    assert_class_string(result, 'Array', 'result should be an array');
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Multiple getAll');

batchgetall_test('out-of-line', (t, connection) => {
  var req = createBatchGetAllRequest(
      t, 'out-of-line', connection, [IDBKeyRange.bound('C', 'E')]);
  req.onsuccess = t.step_func(function(evt) {
    let result = evt.target.result;
    let expected = [[
      {'ch': 'c', 'upper': 'C'}, {'ch': 'd', 'upper': 'D'},
      {'ch': 'e', 'upper': 'E'}
    ]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range');

batchgetall_test('out-of-line', (t, connection) => {
  var req = createBatchGetAllRequest(
      t, 'out-of-line', connection,
      [IDBKeyRange.bound('C', 'M'), IDBKeyRange.bound('O', 'Z')], 3);
  req.onsuccess = t.step_func(function(evt) {
    let result = evt.target.result;
    let expected = [
      [
        {'ch': 'c', 'upper': 'C'}, {'ch': 'd', 'upper': 'D'},
        {'ch': 'e', 'upper': 'E'}
      ],
      [
        {'ch': 'o', 'upper': 'O'}, {'ch': 'p', 'upper': 'P'},
        {'ch': 'q', 'upper': 'Q'}
      ]
    ];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Get bound range with maxCount');

batchgetall_test('out-of-line', (t, connection) => {
  var req = createBatchGetAllRequest(
      t, 'out-of-line', connection, ['Doesn\'t exist1', 'Doesn\'t exist2']);
  req.onsuccess = t.step_func(function(evt) {
    let result = evt.target.result;
    let expected = [[], []];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Non existent key');

batchgetall_test('out-of-line-not-unique', (t, connection) => {
  var req = createBatchGetAllRequest(
      t, 'out-of-line-not-unique', connection, ['first', 'third']);
  req.onsuccess = t.step_func(function(evt) {
    let result = evt.target.result;
    let expected = [
      [
        {'ch': 'a', 'half': 'first'}, {'ch': 'b', 'half': 'first'},
        {'ch': 'c', 'half': 'first'}, {'ch': 'd', 'half': 'first'}
      ],
      [
        {'ch': 'x', 'half': 'third'}, {'ch': 'y', 'half': 'third'},
        {'ch': 'z', 'half': 'third'}
      ]
    ];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Retrieve multiEntry key');

batchgetall_test('out-of-line-multi', (t, connection) => {
  var req =
      createBatchGetAllRequest(t, 'out-of-line-multi', connection, ['vowel']);
  req.onsuccess = t.step_func(function(evt) {
    let result = evt.target.result;
    let expected = [[
      {'ch': 'a', 'attribs': ['vowel', 'first']},
      {'ch': 'e', 'attribs': ['vowel']}, {'ch': 'i', 'attribs': ['vowel']},
      {'ch': 'o', 'attribs': ['vowel']}, {'ch': 'u', 'attribs': ['vowel']}
    ]];
    assertTwoDArrayEquals(result, expected);
    t.done();
  });
}, 'Retrieve one key multiple values');