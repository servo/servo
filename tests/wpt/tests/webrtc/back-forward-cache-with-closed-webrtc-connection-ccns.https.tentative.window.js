// META: title=Testing BFCache support for page with closed WebRTC connection and "Cache-Control: no-store" header.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=resources/webrtc-test-helpers.sub.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
    /*config=*/ { headers: [['Cache-Control', 'no-store']] },
    /*options=*/ { features: 'noopener' }
  );
  // Make sure that we only run the remaining of the test when page with
  // "Cache-Control: no-store" header is eligible for BFCache.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);

  await openThenCloseWebRTC(rc1);
  // The page should not be eligible for BFCache because of the usage
  // of WebRTC.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ false);
  await assertNotRestoredFromBFCache(rc1, [
    'WebRTCSticky',
    'MainResourceHasCacheControlNoStore'
  ]);
});
