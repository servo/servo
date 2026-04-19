// META: title=Testing BFCache support for page with open WebSocket connection, where the WebSocket connection is closed on BFCache.
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
  await openWebSocket(rc1);
  // The page should be eligible for BFCache and the WebSocket connection will
  // be closed when the page is entered into BFCache.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);
  // Read WebSocket event flags
  const { wsError, wsClose } = await readWebSocketCloseAndErrorFlags(rc1);
  assert_true(wsError, 'WebSocket should have error');
  assert_true(wsClose, 'WebSocket should have been closed');
});
