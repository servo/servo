// META: title=IndexedDB: Parallel iteration of cursors in upgradeneeded
// META: global=window,worker
// META: script=resources/support-promises.js
// META: timeout=long

'use strict';

for (const cursorCount of [2, 20, 200, 2000]) {
  promise_test(testCase => {
    return createDatabase(testCase, (database, transaction) => {
             const store =
                 database.createObjectStore('cache', {keyPath: 'key'});
             store.put({key: '42'});

             const promises = [];

             for (let j = 0; j < 2; j += 1) {
               const promise = new Promise((resolve, reject) => {
                 let request = null;
                 for (let i = 0; i < cursorCount / 2; i += 1) {
                   request = store.openCursor();
                 }

                 let continued = false;
                 request.onsuccess = testCase.step_func(() => {
                   const cursor = request.result;

                   if (!continued) {
                     assert_equals(cursor.key, '42');
                     assert_equals(cursor.value.key, '42');
                     continued = true;
                     cursor.continue();
                   } else {
                     assert_equals(cursor, null);
                     resolve();
                   }
                 });
                 request.onerror = () => reject(request.error);
               });
               promises.push(promise);
             }
             return Promise.all(promises);
           }).then(database => {
      database.close();
    });
  }, `${cursorCount} cursors`);
}
