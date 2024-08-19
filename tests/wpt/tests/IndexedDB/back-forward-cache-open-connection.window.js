// META: title=Testing BFCache support for page with open IndexedDB connection, and eviction behavior when receiving versionchange event.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  const prefix = t.name + Math.random();
  const dbname1 = prefix + "_1";
  const dbname2 = prefix + "_2";
  await waitUntilIndexedDBOpenForTesting(rc1, dbname1, 1);
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);

  // The page is ensured to be eligible for BFCache even with open connection,
  // otherwise the previous assertion will fail with PRECONDITION_FAILED.
  // Now we can test if the versionchange event will evict the BFCache.
  await waitUntilIndexedDBOpenForTesting(rc1, dbname2, 1);

  const rc2 = await rc1.navigateToNew();
  // Create an IndexedDB database with higher version.
  // This will fire a version change event on existing connection with the
  // same database name.  The new database will only be opened if the existing
  // connection is closed on receiving the event.
  await waitUntilIndexedDBOpenForTesting(rc2, dbname2, 2);
});
