// META: title=Testing BFCache support for page with open IndexedDB connection, and eviction behavior when receiving versionchange event.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  await createIndexedDBForTesting(rc1, 'test_idb', 1);
  await assertBFCache(rc1, /*shouldRestoreFromBFCache=*/ true);

  // The page is ensured to be eligible for BFCache even with open connection,
  // otherwise the previous assertion will fail with PRECONDITION_FAILED.
  // Now we can test if the versionchange event will evict the BFCache.
  await createIndexedDBForTesting(rc1, 'test_idb_2', 1);

  const rc2 = await rc1.navigateToNew();
  // Create an IndexedDB database with higher version.
  await createIndexedDBForTesting(rc2, 'test_idb_2', 2);
  await rc2.historyBack();
  // The previous page receiving versionchange event should be evicted.
  await assert_not_bfcached(rc1);
});
