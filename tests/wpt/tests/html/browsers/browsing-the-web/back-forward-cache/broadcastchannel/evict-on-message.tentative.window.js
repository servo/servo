// META: title=BroadcastChannel message while in bfcache should evict the entry.
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
      /*extraConfig=*/ {}, /*options=*/ {features: 'noopener'});
  await rc1.executeScript(() => {
    const channel = new BroadcastChannel('foo');
    channel.addEventListener('message', event => {
      channel.postMessage('Message received: ' + event.data);
    });
  });
  await prepareForBFCache(rc1);
  const newRemoteContextHelper = await rc1.navigateToNew();
  await assertSimplestScriptRuns(newRemoteContextHelper);

  // Post a message to a channel in bfcache. This should trigger eviction.
  const channel = new BroadcastChannel('foo');  // Access shared channel
  channel.postMessage('Sending a message should evict a bfcache entry.');

  await newRemoteContextHelper.historyBack();

  // It's possible that the pages with open broadcastchannel are not allowed
  // into bfcache. Set preconditionFailReasons to catch that case. Otherwise
  // expect the eviction reason.
  await assertNotRestoredFromBFCache(
      rc1, ['broadcastchannel-message'],
      /*preconditonFailReasons=*/['broadcastchannel']);
});
