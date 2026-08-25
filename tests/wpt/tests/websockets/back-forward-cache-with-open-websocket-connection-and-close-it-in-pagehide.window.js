// META: title=Testing BFCache support for a page with an open WebSocket connection, but close it in pagehide.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/websockets/constants.sub.js
// META: script=resources/websockets-test-helpers.sub.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();
    // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ { features: 'noopener' });

  await openWebSocketAndCloseItInPageHide(rc1);

  // The page should be eligible for BFCache because the WebSocket connection will be closed in `pagehide`.
  // `pagehide` is dispatched before BFCache
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);

  // Read WebSocket event flags
  const { wsError, wsClose } = await readWebSocketCloseAndErrorFlags(rc1);

  assert_false(wsError, 'WebSocket should not have error');
  assert_true(wsClose, 'WebSocket should have been closed via pagehide');
});
