// META: global=window,dedicatedworker,sharedworker,serviceworker
// META: script=../support-promises.js
// META: script=./reading-autoincrement-common.js

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_id');

  const result = await getAllViaCursor(testCase, index);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, i, 'Autoincrement index key');
    assert_equals(result[i - 1].primaryKey, i, 'Autoincrement primary key');
    assert_equals(result[i - 1].value.id, i, 'Autoincrement key in value');
    assert_equals(result[i - 1].value.name, nameForId(i),
                  'String property in value');
  }

  database.close();
}, 'IDBIndex.openCursor() iterates over an index on the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_id');

  const result = await getAllKeysViaCursor(testCase, index);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, i, 'Autoincrement index key');
    assert_equals(result[i - 1].primaryKey, i, 'Autoincrement primary key');
  }

  database.close();
}, 'IDBIndex.openKeyCursor() iterates over an index on the autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_name');

  const stringSortedIds = idsSortedByStringCompare();

  const result = await getAllViaCursor(testCase, index);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, nameForId(stringSortedIds[i - 1]),
                  'Index key');
    assert_equals(result[i - 1].primaryKey, stringSortedIds[i - 1],
                  'Autoincrement primary key');
    assert_equals(result[i - 1].value.id, stringSortedIds[i - 1],
                  'Autoincrement key in value');
    assert_equals(result[i - 1].value.name, nameForId(stringSortedIds[i - 1]),
                  'String property in value');
  }

  database.close();
}, 'IDBIndex.openCursor() iterates over an index not covering the ' +
   'autoincrement key');

promise_test(async testCase => {
  const database = await setupAutoincrementDatabase(testCase);

  const transaction = database.transaction(['store'], 'readonly');
  const store = transaction.objectStore('store');
  const index = store.index('by_name');

  const stringSortedIds = idsSortedByStringCompare();

  const result = await getAllKeysViaCursor(testCase, index);
  assert_equals(result.length, 32);
  for (let i = 1; i <= 32; ++i) {
    assert_equals(result[i - 1].key, nameForId(stringSortedIds[i - 1]),
                  'Index key');
    assert_equals(result[i - 1].primaryKey, stringSortedIds[i - 1],
                  'Autoincrement primary key');
  }

  database.close();
}, 'IDBIndex.openKeyCursor() iterates over an index not covering the ' +
   'autoincrement key');