// META: title=Index Tombstones
// META: script=resources/support-promises.js

// This test is used to trigger a special case in Chrome with how it deals with
// index creation & modification. This had caused issues before.
// See https://crbug.com/1033996

async function iterateAndReturnAllCursorResult(testCase, cursorRequest) {
  return new Promise((resolve, reject) => {
    let results = [];
    cursorRequest.onsuccess = testCase.step_func(function(event) {
      const cursor = event.target.result;
      if (!cursor) {
        resolve(results);
        return;
      }
      results.push(cursor.value);
      cursor.continue();
    });
    cursorRequest.onerror = reject;
  });
}

async function createTombstones(testCase, db) {
  const txn1 = db.transaction(['objectStore'], 'readwrite');
  txn1.objectStore('objectStore').add({key: 'firstItem', indexedOn: 1});
  txn1.objectStore('objectStore').add({key: 'secondItem', indexedOn: 2});
  txn1.objectStore('objectStore').add({key: 'thirdItem', indexedOn: 3});
  const txn2 = db.transaction(['objectStore'], 'readwrite');
  txn2.objectStore('objectStore').put({key: 'firstItem', indexedOn: -10});
  txn2.objectStore('objectStore').put({key: 'secondItem', indexedOn: 4});
  txn2.objectStore('objectStore').put({key: 'thirdItem', indexedOn: 10});
  await promiseForTransaction(testCase, txn1);
  await promiseForTransaction(testCase, txn2);
}

async function run_test(testCase, transactionMode, direction) {
  const db = await createDatabase(testCase, db => {
    db.createObjectStore('objectStore', {keyPath: 'key'})
        .createIndex('index', 'indexedOn');
  });
  await createTombstones(testCase, db);

  const txn = db.transaction(['objectStore'], transactionMode);
  cursor = txn.objectStore('objectStore').index('index').openCursor(
      IDBKeyRange.bound(-11, 11), direction);
  let results = await iterateAndReturnAllCursorResult(testCase, cursor);
  assert_equals(results.length, 3);
  db.close();
}

promise_test(async testCase => {
  await run_test(testCase, 'readonly', 'next');
}, 'Forward iteration over an index in a readonly transaction');

promise_test(async testCase => {
  await run_test(testCase, 'readonly', 'prev');
}, 'Backward iteration over an index in a readonly transaction');

promise_test(async testCase => {
  await run_test(testCase, 'readwrite', 'next');
}, 'Forward iteration over an index in a readwrite transaction');

promise_test(async testCase => {
  await run_test(testCase, 'readwrite', 'prev');
}, 'Backward iteration over an index in a readwrite transaction');
