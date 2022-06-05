// META: global=window,dedicatedworker,sharedworker,serviceworker
// META: script=resources/support-promises.js
// META: script=resources/reading-autoincrement-common.js

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');

  const result = await getAllViaCursor(testCase, store);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, i, 'Autoincrement key');
    assert_equals(result[i - 1].primaryKey, i, 'Autoincrement primary key');
    assert_equals(result[i - 1].value.id, i, 'Autoincrement key in value');
    assert_equals(result[i - 1].value.name, nameForId(i),
                  'string property in value');
  }

  database.close();
}, 'IDBObjectStore.openCursor() iterates over an autoincrement store');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');

  const result = await getAllKeysViaCursor(testCase, store);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, i, 'Incorrect autoincrement key');
    assert_equals(result[i - 1].primaryKey, i, 'Incorrect primary key');
  }

  database.close();
}, 'IDBObjectStore.openKeyCursor() iterates over an autoincrement store');
