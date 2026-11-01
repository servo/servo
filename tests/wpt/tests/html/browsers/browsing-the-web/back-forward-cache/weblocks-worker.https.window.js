// META: title=WebLocks granted to a dedicated worker in BFCache should be deferred.
// META: timeout=long
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper-tests/resources/test-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();
  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*extraConfig=*/ {}, /*options=*/ { features: 'noopener' });

  const lockName = `bfcache_weblock_test_${token()}`;
  const channelName = `bfcache_lock_test_channel_${token()}`;

  // BroadcastChannel to receive the grant notification from worker.
  const bc = new BroadcastChannel(channelName);
  t.add_cleanup(() => bc.close());
  let grantedReceived = false;
  const grantedPromise = new Promise(resolve => {
    bc.onmessage = t.step_func(e => {
      if (e.data === 'granted') {
        grantedReceived = true;
        resolve();
      }
    });
  });

  // Acquire the lock in the main test window first.
  let mainLockResolver;
  let releaseMainLock;
  const mainLockPromise = new Promise(resolve => { mainLockResolver = resolve; });
  navigator.locks.request(lockName, async lock => {
    mainLockResolver();
    // Keep holding the lock until we decide to release it.
    await new Promise(resolve => {
      t.add_cleanup(resolve);
      releaseMainLock = resolve;
    });
  });
  await mainLockPromise;

  // Start a worker in the remote context.
  const worker = await rc1.addWorker(
      /*workerVar=*/ undefined,
    {
      scripts: [],
    }
  );

  // In the worker, request the same lock.
  // Since the main window holds it, this will pend.
  await worker.executeScript((lockName, channelName) => {
    navigator.locks.request(lockName, () => {
      const bc = new BroadcastChannel(channelName);
      bc.postMessage('granted');
      bc.close();
    });
  }, [lockName, channelName]);

  // Prepare rc1 for BFCache.
  await prepareForBFCache(rc1);

  // Navigate away.
  const rc1Away = await rc1.navigateToNew();
  await assertSimplestScriptRuns(rc1Away);

  // Now rc1 and its worker should be in BFCache.
  // Wait a bit to ensure they are frozen.
  await new Promise(resolve => t.step_timeout(resolve, 1000));

  // Release the lock in the main window.
  // If the worker is correctly frozen, it will acquire the lock but will not run the callback.
  releaseMainLock();

  // Wait to see if the worker does not run the callback.
  await new Promise(resolve => t.step_timeout(resolve, 1000));

  // Assert that we have not received the message yet.
  assert_false(grantedReceived, "Lock should not be granted while in BFCache");

  // Restore rc1 from BFCache.
  await rc1Away.historyBack();
  await assertImplementsBFCacheOptional(rc1);

  // Now that it is restored, the worker should unfreeze and run the callback.
  // Wait for the message to arrive.
  await grantedPromise;

  assert_true(grantedReceived, "Lock should be granted after restore");

}, 'WebLocks grant to worker in BFCache is deferred');
