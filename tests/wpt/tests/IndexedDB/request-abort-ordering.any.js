// META: title=IndexedDB: request abort events are delivered in order
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/support.js

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

promise_test(testCase => {
  let requests;

  return createDatabase(
             testCase,
             (database, transaction) => {
               createBooksStore(testCase, database);
             })
      .then(database => {
        const transaction = database.transaction(['books'], 'readwrite');
        const store = transaction.objectStore('books');
        const index = store.index('by_author');
        const cursorRequest = store.openCursor(IDBKeyRange.lowerBound(0));

        return new Promise((resolve, reject) => {
          cursorRequest.onerror = testCase.step_func(event => {
            event.preventDefault();
            reject(cursorRequest.error);
          });

          cursorRequest.onsuccess = testCase.step_func(() => {
            const cursor = cursorRequest.result;
            requests = [
              () => store.get(123456),
              () => index.get('Fred'),
              () => store.count(),
              () => index.count(),
              () =>
                  store.put({title: 'Bedrock II', author: 'Barney', isbn: 987}),
              () => store.getAll(),
              () => index.getAll(),
              () => store.get(999999),
              () => index.get('Nobody'),
              () => store.openCursor(IDBKeyRange.lowerBound(0)),
              () => index.openCursor(IDBKeyRange.lowerBound('')),
              () => {
                cursor.continue();
                return cursorRequest;
              },
            ];

            const results = [];
            const promises = [];
            for (let i = 0; i < requests.length; ++i) {
              promises.push(new Promise((resolve, reject) => {
                const requestId = i;
                const request = requests[i](store);
                request.onsuccess = testCase.step_func(() => {
                  reject(new Error(
                      'IDB requests should not succeed after transaction abort'));
                });
                request.onerror = testCase.step_func(event => {
                  event.preventDefault();
                  results.push([requestId, request.error]);
                  resolve();
                });
              }));
            };
            transaction.abort();
            resolve(Promise.all(promises).then(() => results));
          });
        });
      })
      .then(results => {
        assert_equals(
            results.length, requests.length,
            'Promise.all should resolve after all sub-promises resolve');
        for (let i = 0; i < requests.length; ++i) {
          assert_equals(
              results[i][0], i, 'error event order should match request order');
          assert_equals(
              results[i][1].name, 'AbortError',
              'transaction aborting should result in AbortError on all requests');
        }
      });
});
