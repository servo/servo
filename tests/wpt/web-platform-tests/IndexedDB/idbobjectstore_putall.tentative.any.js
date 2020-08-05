// META: script=support-promises.js

promise_test(async testCase => {
  const db = await createDatabase(testCase, db => {
    const store = createBooksStore(testCase, db);
    let values = [
      {isbn: 'one', title: 'title1'},
      {isbn: 'two', title: 'title2'},
      {isbn: 'three', title: 'title3'}
    ];
    const putAllRequests = store.putAll(values);
    putAllRequests.forEach(async request => {
      await promiseForRequest(testCase, request);
    });
  });

  const txn = db.transaction(['books'], 'readonly');
  const objectStore = txn.objectStore('books');
  const getRequest1 = objectStore.get('one');
  const getRequest2 = objectStore.get('two');
  const getRequest3 = objectStore.get('three');
  await promiseForTransaction(testCase, txn);
  assert_array_equals(
      [getRequest1.result.title,
          getRequest2.result.title,
          getRequest3.result.title],
      ['title1', 'title2', 'title3'],
      'All three retrieved titles should match those that were put.');
  db.close();
}, 'Data can be successfully inputted into an object store using putAll.');