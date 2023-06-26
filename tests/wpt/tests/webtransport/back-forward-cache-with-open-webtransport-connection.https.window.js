// META: title=Testing BFCache support for page with open WebTransport connection.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=resources/webtransport-test-helpers.sub.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ { features: 'noopener' });
  await openWebTransport(rc1);
  // The page should be eligible for BFCache and the WebTransport connection
  // should be closed.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);
  await rc1.executeScript(async () => {
    assert_false(window.testWebTransport === undefined);
    await window.testWebTransport.closed;
  });
});
