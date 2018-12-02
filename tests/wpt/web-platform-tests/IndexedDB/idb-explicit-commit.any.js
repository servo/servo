// META: script=support-promises.js

/**
 * This file contains the webplatform tests for the explicit commit() method
 * of the IndexedDB transaction API.
 *
 * @author andreasbutler@google.com
 */

promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  objectStore.put({isbn: 'one', title: 'title1'});
  objectStore.put({isbn: 'two', title: 'title2'});
  objectStore.put({isbn: 'three', title: 'title3'});
  txn.commit();
  await promiseForTransaction(testCase, txn);

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequestitle1 = objectStore2.get('one');
  const getRequestitle2 = objectStore2.get('two');
  const getRequestitle3 = objectStore2.get('three');
  txn2.commit();
  await promiseForTransaction(testCase, txn2);
  assert_array_equals(
      [getRequestitle1.result.title,
          getRequestitle2.result.title,
          getRequestitle3.result.title],
      ['title1', 'title2', 'title3'],
      'All three retrieved titles should match those that were put.');
  db.close();
}, 'Explicitly committed data can be read back out.');


promise_test(async testCase => {
  let db = await createDatabase(testCase, () => {});
  assert_equals(1, db.version, 'A database should be created as version 1');
  db.close();

  // Upgrade the versionDB database and explicitly commit its versionchange
  // transaction.
  db = await migrateDatabase(testCase, 2, async (db, txn) => {
    txn.commit();
  });
  assert_equals(2, db.version,
      'The database version should have been incremented regardless of '
      + 'whether the versionchange transaction was explicitly or implicitly '
      + 'committed.');
  db.close();
}, 'commit() on a version change transaction does not cause errors.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  txn.commit();
  assert_throws('TransactionInactiveError',
      () => { objectStore.put({isbn: 'one', title: 'title1'}); },
      'After commit is called, the transaction should be inactive.');
  db.close();
}, 'A committed transaction becomes inactive immediately.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const putRequest = objectStore.put({isbn: 'one', title: 'title1'});
  putRequest.onsuccess = testCase.step_func(() => {
    assert_throws('TransactionInactiveError',
      () => { objectStore.put({isbn:'two', title:'title2'}); },
      'The transaction should not be active in the callback of a request after '
      + 'commit() is called.');
  });
  txn.commit();
  await promiseForTransaction(testCase, txn);
  db.close();
}, 'A committed transaction is inactive in future request callbacks.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  txn.commit();

  assert_throws('TransactionInactiveError',
      () => { objectStore.put({isbn:'one', title:'title1'}); },
      'After commit is called, the transaction should be inactive.');

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest = objectStore2.get('one');
  await promiseForTransaction(testCase, txn2);
  assert_equals(getRequest.result, undefined);

  db.close();
}, 'Puts issued after commit are not fulfilled.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  txn.abort();
  assert_throws('InvalidStateError',
      () => { txn.commit(); },
      'The transaction should have been aborted.');
  db.close();
}, 'Calling commit on an aborted transaction throws.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  txn.commit();
  assert_throws('InvalidStateError',
      () => { txn.commit(); },
      'The transaction should have already committed.');
  db.close();
}, 'Calling commit on a committed transaction throws.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const putRequest = objectStore.put({isbn:'one', title:'title1'});
  txn.commit();
  assert_throws('InvalidStateError',
      () => { txn.abort(); },
      'The transaction should already have committed.');
  const txn2 = db.transaction(['books'], 'readwrite');
  const objectStore2 = txn2.objectStore('books');
  const getRequest = objectStore2.get('one');
  await promiseForTransaction(testCase, txn2);
  assert_equals(
      getRequest.result.title,
      'title1',
      'Explicitly committed data should be gettable.');
  db.close();
}, 'Calling abort on a committed transaction throws and does not prevent '
   + 'persisting the data.');


promise_test(async testCase => {
  const db = await createDatabase(testCase, async db => {
    await createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const releaseTxnFunction = keepAlive(testCase, txn, 'books');

  // Break up the scope of execution to force the transaction into an inactive
  // state.
  await timeoutPromise(0);

  assert_throws('InvalidStateError',
      () => { txn.commit(); },
      'The transaction should be inactive so calling commit should throw.');
  releaseTxnFunction();
  db.close();
}, 'Calling txn.commit() when txn is inactive should throw.');
