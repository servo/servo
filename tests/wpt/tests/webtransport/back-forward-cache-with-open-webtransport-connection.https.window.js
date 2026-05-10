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
      /*config=*/ {scripts: ['/resources/testharness.js']},
      /*options=*/ {features: 'noopener'});
  await openWebTransport(rc1);
  // The page should be eligible for BFCache and the WebTransport connection
  // should be closed.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);
  await rc1.executeScript(async () => {
    assert_false(window.testWebTransport === undefined);
    try {
      await window.testWebTransport.closed;
      // The promise should reject because BFCache entry terminates the
      // connection.
      assert_unreached('The WebTransport closed promise should reject.');
    } catch (e) {
      assert_equals(
          e.source, 'session', 'The error source should be \'session\'');
    }
  });
});
