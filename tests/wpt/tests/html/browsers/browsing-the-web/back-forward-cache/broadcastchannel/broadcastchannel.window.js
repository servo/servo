// META: title=Ensure that open broadcastchannel does not block bfcache.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper-tests/resources/test-helper.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();
  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*extraConfig=*/ {
        origin: 'HTTP_ORIGIN',
        scripts: [],
        headers: [],
      },
      /*options=*/ {features: 'noopener'});
  await rc1.executeScript(() => {
    window.foo = new BroadcastChannel('foo');
  });
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBfcache=*/ true);
});
