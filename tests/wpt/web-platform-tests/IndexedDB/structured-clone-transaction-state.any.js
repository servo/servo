// META: script=support-promises.js
// META: title=Indexed DB transaction state during Structured Serializing
// META: timeout=long
'use strict';

promise_test(async testCase => {
  const db = await createDatabase(testCase, database => {
    database.createObjectStore('store');
  });

  const transaction = db.transaction(['store'], 'readwrite');
  const objectStore = transaction.objectStore('store');

  let getterCalled = false;
  const activeValue = {};
  Object.defineProperty(activeValue, 'propertyName', {
    enumerable: true,
    get: testCase.step_func(() => {
      getterCalled = true;
      assert_throws('TransactionInactiveError', () => {
        objectStore.get('key');
      }, 'transaction should not be active during structured clone');
      return 'value that should not be used';
    }),
  });
  objectStore.add(activeValue, 'key');
  await promiseForTransaction(testCase, transaction);
  db.close();

  assert_true(getterCalled,
              "activeValue's getter should be called during test");
}, 'Transaction inactive during structured clone in IDBObjectStore.add()');

promise_test(async testCase => {
  const db = await createDatabase(testCase, database => {
    database.createObjectStore('store');
  });

  const transaction = db.transaction(['store'], 'readwrite');
  const objectStore = transaction.objectStore('store');

  let getterCalled = false;
  const activeValue = {};
  Object.defineProperty(activeValue, 'propertyName', {
    enumerable: true,
    get: testCase.step_func(() => {
      getterCalled = true;
      assert_throws('TransactionInactiveError', () => {
        objectStore.get('key');
      }, 'transaction should not be active during structured clone');
      return 'value that should not be used';
    }),
  });

  objectStore.put(activeValue, 'key');
  await promiseForTransaction(testCase, transaction);
  db.close();

  assert_true(getterCalled,
              "activeValue's getter should be called during test");
}, 'Transaction inactive during structured clone in IDBObjectStore.put()');

promise_test(async testCase => {
  const db = await createDatabase(testCase, database => {
    const objectStore = database.createObjectStore('store');
    objectStore.put({}, 'key');
  });

  const transaction = db.transaction(['store'], 'readwrite');
  const objectStore = transaction.objectStore('store');

  let getterCalled = false;
  const activeValue = {};
  Object.defineProperty(activeValue, 'propertyName', {
    enumerable: true,
    get: testCase.step_func(() => {
      getterCalled = true;
      assert_throws('TransactionInactiveError', () => {
        objectStore.get('key');
      }, 'transaction should not be active during structured clone');
      return 'value that should not be used';
    }),
  });

  const request = objectStore.openCursor();
  request.onsuccess = testCase.step_func(() => {
    const cursor = request.result;
    cursor.update(activeValue);
  });

  await promiseForTransaction(testCase, transaction);
  db.close();

  assert_true(getterCalled,
              "activeValue's getter should be called during test");
}, 'Transaction inactive during structured clone in IDBCursor.update()');
