// META: global=window,dedicatedworker,sharedworker,serviceworker
// META: script=../support-promises.js
// META: script=./reading-autoincrement-common.js

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const request = store.getAll();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].id, i, 'Autoincrement key');
    assert_equals(result[i - 1].name, nameForId(i), 'String property');
  }

  database.close();
}, 'IDBObjectStore.getAll() for an autoincrement store');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const request = store.getAllKeys();
  const result = await promiseForRequest(testCase, request);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i)
    assert_equals(result[i - 1], i, 'Autoincrement key');

  database.close();
}, 'IDBObjectStore.getAllKeys() for an autoincrement store');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');

  for (let i = 1; i <= 32; ++i) {
    const request = store.get(i);
    const result = await promiseForRequest(testCase, request);
    assert_equals(result.id, i, 'Autoincrement key');
    assert_equals(result.name, nameForId(i), 'String property');
  }

  database.close();
}, 'IDBObjectStore.get() for an autoincrement store');
