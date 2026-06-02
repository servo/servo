// META: title=BFCache support test for page with open IndexedDB transaction
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  const dbname = t.name + Math.random();
  await rc1.executeScript((dbname) => {
    return new Promise(resolve => {
      // Create an IndexedDB database and the object store named `store` as the
      // test scope for the transaction later on.
      const db = indexedDB.open(/*name=*/ dbname, /*version=*/ 1);
      db.onupgradeneeded = () => {
        db.result.createObjectStore('store');
        addEventListener('pagehide', () => {
          let transaction = db.result.transaction(['store'], 'readwrite');
          let store = transaction.objectStore('store');
          store.put('key', 'value');

          // Queue a request to close the connection, while keeping the transaction
          // open, so that the BFCache eligibility will be determined solely by the
          // pending transaction.
          db.result.close();
        });
        // Only resolve the promise when the connection is established
        // and the `pagehide` event listener is added.
        resolve();
      };
    });
  }, [dbname]);

  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);
});
