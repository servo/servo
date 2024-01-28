// META: title=Testing BFCache support for page with open WebRTC connection and live MediaStreamTrack.
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
      /*config=*/ null, /*options=*/ { features: 'noopener' });
  await openWebRTC(rc1);
  // The page should not be eligible for BFCache because of open WebRTC connection and live MediaStreamTrack.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ false);
  await assertNotRestoredFromBFCache(rc1, ['WebRTC', 'LiveMediaStreamTrack']);
});
