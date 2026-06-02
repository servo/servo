// META: title=BroadcastChannel messages dispatched to dedicated worker in bfcache should be queued.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper-tests/resources/test-helper.js

'use strict';

// Ensure that broadcast channel messages sent to a dedicated
// worker in bfcache are queued and dispatched upon restore.
promise_test(async t => {
  const rcHelper = new RemoteContextHelper();
  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*extraConfig=*/ {}, /*options=*/ {features: 'noopener'});
  let workerVar;
  const worker = await rc1.addWorker(
      workerVar,
      {
        scripts: ['../resources/worker-with-broadcastchannel.js'],
      },
  );
  await assertSimplestScriptRuns(worker);

  await prepareForBFCache(rc1);
  const newRemoteContextHelper = await rc1.navigateToNew();
  await assertSimplestScriptRuns(newRemoteContextHelper);

  // Send a message to a dedicated worker in bfcache.
  let channel = new BroadcastChannel('foo');
  channel.postMessage('bar');

  await newRemoteContextHelper.historyBack();
  // Make sure that rc1 gets restored without getting evicted. Messages
  // while in bfcache should be queued.
  await assertImplementsBFCacheOptional(rc1);

  // A message should arrive upon bfcache restore.
  await worker.executeScript(() => {
    return waitForEventsPromise(1);
  });
});
