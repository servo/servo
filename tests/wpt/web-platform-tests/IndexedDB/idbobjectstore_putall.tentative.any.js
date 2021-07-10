// META: script=support-promises.js

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const values = [
    {isbn: 'one', title: 'title1'},
    {isbn: 'two', title: 'title2'},
    {isbn: 'three', title: 'title3'}
  ];
  const putAllRequest = objectStore.putAllValues(values);
  // TODO(nums): Check that correct keys are returned.
  await promiseForRequest(testCase, putAllRequest);
  await promiseForTransaction(testCase, txn);

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest1 = objectStore2.get('one');
  const getRequest2 = objectStore2.get('two');
  const getRequest3 = objectStore2.get('three');
  await promiseForTransaction(testCase, txn2);
  assert_array_equals(
      [getRequest1.result.title,
          getRequest2.result.title,
          getRequest3.result.title],
      ['title1', 'title2', 'title3'],
      'All three retrieved titles should match those that were put.');
  db.close();
}, 'Data can be successfully inserted into an object store using putAll.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const values = [
    {isbn: ['one', 'two', 'three'], title: 'title1'},
    {isbn: ['four', 'five', 'six'], title: 'title2'},
    {isbn: ['seven', 'eight', 'nine'], title: 'title3'}
  ];
  const putAllRequest = objectStore.putAllValues(values);
  // TODO(nums): Check that correct keys are returned.
  await promiseForRequest(testCase, putAllRequest);
  await promiseForTransaction(testCase, txn);

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest1 = objectStore2.get(['one', 'two', 'three']);
  const getRequest2 = objectStore2.get(['four', 'five', 'six']);
  const getRequest3 = objectStore2.get(['seven', 'eight', 'nine']);
  await promiseForTransaction(testCase, txn2);
  assert_array_equals(
      [getRequest1.result.title,
          getRequest2.result.title,
          getRequest3.result.title],
      ['title1', 'title2', 'title3'],
      'All three retrieved titles should match those that were put.');
  db.close();
}, 'Values with array keys can be successfully inserted into an object'
    + ' store using putAll.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const putAllRequest = objectStore.putAllValues([]);
  await promiseForRequest(testCase, putAllRequest);
  await promiseForTransaction(testCase, txn);
  // TODO(nums): Check that an empty key array is returned.
  db.close();
}, 'Inserting an empty list using putAll.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const putAllRequest = objectStore.putAllValues([{}, {}, {}]);
  // TODO(nums): Check that correct keys are returned.
  await promiseForRequest(testCase, putAllRequest);
  await promiseForTransaction(testCase, txn);

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest1 = objectStore2.get(1);
  const getRequest2 = objectStore2.get(2);
  const getRequest3 = objectStore2.get(3);
  await Promise.all([
    promiseForRequest(testCase, getRequest1),
    promiseForRequest(testCase, getRequest2),
    promiseForRequest(testCase, getRequest3),
  ]);
  db.close();
}, 'Empty values can be inserted into an objectstore'
    + ' with a key generator using putAll.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readonly');
  const objectStore = txn.objectStore('books');
  assert_throws_dom('ReadOnlyError',
    () => { objectStore.putAllValues([{}]); },
    'The transaction is readonly');
  db.close();
}, 'Attempting to insert with a read only transaction using putAll throws a '
    + 'ReadOnlyError.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const putRequest = await objectStore.put({isbn: 1, title: "duplicate"});
  await promiseForRequest(testCase, putRequest);
  const putAllRequest = objectStore.putAllValues([
    {isbn: 2, title: "duplicate"},
    {isbn: 3, title: "duplicate"}
  ]);
  const errorEvent = await requestWatcher(testCase,
                                        putAllRequest).wait_for('error');
  assert_equals(errorEvent.target.error.name, "ConstraintError");
  errorEvent.preventDefault();
  // The transaction still receives the error event even though it
  // isn't aborted.
  await transactionWatcher(testCase, txn).wait_for(['error', 'complete']);

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest1 = objectStore2.get(1);
  const getRequest2 = objectStore2.get(2);
  const getRequest3 = objectStore2.get(3);
  await promiseForTransaction(testCase, txn2);
  assert_array_equals(
      [getRequest1.result.title, getRequest2.result, getRequest3.result],
      ["duplicate", undefined, undefined],
      'None of the values should have been inserted.');
  db.close();
}, 'Inserting duplicate unique keys into a store that already has the key'
    + 'using putAll throws a ConstraintError.');

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStoreWithoutAutoIncrement(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  const values = [
    {title: "title1", isbn: 1},
    {title: "title2"}
  ];
  assert_throws_dom('DataError',
    () => { const putAllRequest = objectStore.putAllValues(values); },
    "Evaluating the object store's key path did not yield a value");

  const txn2 = db.transaction(['books'], 'readonly');
  const objectStore2 = txn2.objectStore('books');
  const getRequest1 = objectStore2.get(1);
  const getRequest2 = objectStore2.get(2);
  await promiseForTransaction(testCase, txn2);
  assert_array_equals(
      [getRequest1.result, getRequest2.result],
      [undefined, undefined],
      'No data should have been inserted');
  db.close();
}, 'Inserting values without the key into an object store that'
    + ' does not have generated keys throws an exception.');

// TODO(nums): Add test for insertion into multi entry indexes
// TODO(nums): Add test for inserting unique keys into a store
// that doesn't already have the key https://crbug.com/1115649
