// META: title=BFCache support test for page with open IndexedDB connection
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  await rc1.executeScript(() => {
    // Create an IndexedDB database.
    const db = indexedDB.open(/*name=*/ 'test_idb', /*version=*/ 1);
    db.onupgradeneeded = () => {
      db.result.createObjectStore('store');
    };
  });

  await assertBFCache(rc1, /*shouldRestoreFromBFCache=*/ true);
});
