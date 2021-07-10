// META: global=window,dedicatedworker,sharedworker,serviceworker
// META: script=../support-promises.js
// META: script=./reading-autoincrement-common.js

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_id');
  const request = index.getAll();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].id, i, 'Autoincrement key');
    assert_equals(result[i - 1].name, nameForId(i), 'String property');
  }

  database.close();
}, 'IDBIndex.getAll() for an index on the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_id');
  const request = index.getAllKeys();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i)
    assert_equals(result[i - 1], i, 'Autoincrement key');

  database.close();
}, 'IDBIndex.getAllKeys() for an index on the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_id');

  for (let i = 1; i <= 32; ++i) {
    const request = index.get(i);
    const result = await promiseForRequest(testCase, request);
    assert_equals(result.id, i, 'autoincrement key');
    assert_equals(result.name, nameForId(i), 'string property');
  }

  database.close();
}, 'IDBIndex.get() for an index on the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const stringSortedIds = idsSortedByStringCompare();

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_name');
  const request = index.getAll();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].id, stringSortedIds[i - 1],
                  'autoincrement key');
    assert_equals(result[i - 1].name, nameForId(stringSortedIds[i - 1]),
                  'string property');
  }

  database.close();
}, 'IDBIndex.getAll() for an index not covering the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const stringSortedIds = idsSortedByStringCompare();

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_name');
  const request = index.getAllKeys();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i)
    assert_equals(result[i - 1], stringSortedIds[i - 1], 'String property');

  database.close();
}, 'IDBIndex.getAllKeys() returns correct result for an index not covering ' +
   'the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_name');

  for (let i = 1; i <= 32; ++i) {
    const request = index.get(nameForId(i));
    const result = await promiseForRequest(testCase, request);
    assert_equals(result.id, i, 'Autoincrement key');
    assert_equals(result.name, nameForId(i), 'String property');
  }

  database.close();
}, 'IDBIndex.get() for an index not covering the autoincrement key');
