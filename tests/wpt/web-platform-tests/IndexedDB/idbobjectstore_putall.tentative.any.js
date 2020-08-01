// META: script=support-promises.js

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
  });
  const txn = db.transaction(['books'], 'readwrite');
  const objectStore = txn.objectStore('books');
  let values = [
    {isbn: 'one', title: 'title1'},
    {isbn: 'two', title: 'title2'},
    {isbn: 'three', title: 'title3'}
  ];
  let putAllRequest = objectStore.putAll(values);
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
}, 'Data can be successfully inputted into an object store using putAll.');
