// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/test-helpers.js
// META: script=resources/messaging-helpers.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: timeout=long

'use strict';


createBFCacheTest(async (t, testControls) => {
  const {getRemoteFuncs, assertBFCacheEligibility} = testControls;
  const [createAndReleaseWFS] = getRemoteFuncs('createAndReleaseWFS');

  for (const mode of WFS_MODES) {
    await createAndReleaseWFS(mode, 'hello.txt');
    await assertBFCacheEligibility(/*shouldRestoreFromBFCache=*/ true);
  }
}, 'Creating an WFS should not make it ineligible for the BFCache.');

createBFCacheTest(async (t, testControls) => {
  const origFile = 'hello.txt';
  const diffFile = 'world.txt';

  const {getRemoteFuncs, forward, back} = testControls;
  const [createWFS, releaseWFS, createAndReleaseWFS] =
      getRemoteFuncs('createWFS', 'releaseWFS', 'createAndReleaseWFS');

  async function testTakeLockOnForward(
      mode, fileName, shouldRestoreFromBFCache) {
    await forward();

    assert_equals(
        await createAndReleaseWFS(mode, fileName), shouldRestoreFromBFCache);

    await back(shouldRestoreFromBFCache);
  }

  for (const backMode of WFS_MODES) {
    for (const forwMode of WFS_MODES) {
      const contentiousLocks = wfsModesAreContentious(backMode, forwMode);

      // Create a lock on the page that will be BFCached.
      const lockId = await createWFS(backMode, origFile);
      assert_true(lockId !== undefined);

      // Navigating to a new page and taking a lock on a different file should
      // not evict the page from BFCache.
      await testTakeLockOnForward(
          forwMode, diffFile, /*shouldRestoreFromBFCache=*/ true);

      // Navigating to a new page and taking a lock on the same file should only
      // evict if the locks are contentious.
      await testTakeLockOnForward(
          forwMode, origFile, /*shouldRestoreFromBFCache=*/ !contentiousLocks);

      // Release the lock when there isn't contention since it won't have been
      // evicted.
      if (!contentiousLocks) {
        await releaseWFS(lockId);
      }
    }
  }
}, `Creating a WFS on an active page evicts an inactive page on contention.`)
