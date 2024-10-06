// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long

"use strict;"

promise_test(async test => {
  const uuid = generateUuid(test);

  // To minimize the risk of the auction completing before the abort signal,
  // make the bid script hand, and increase the per-buyer script timeout.
  await joinInterestGroup(
      test, uuid,
      createBiddingScriptURL({generateBid: 'while(1);'}));

  let abortController = new AbortController();
  let promise = runBasicFledgeAuction(
      test, uuid,
      { signal: abortController.signal,
        perBuyerTimeouts: {'*': 1000}
      });
  abortController.abort('reason');
  try {
    await promise;
  } catch (e) {
    assert_equals(e, 'reason');
    return;
  }
  throw 'Exception unexpectedly not thrown';
}, 'Abort auction.');

promise_test(async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(test, uuid);

  let abortController = new AbortController();
  abortController.abort('reason');
  try {
    await runBasicFledgeAuction(test, uuid, {signal: abortController.signal});
  } catch (e) {
    assert_equals(e, 'reason');
    return;
  }
  throw 'Exception unexpectedly not thrown';
}, 'Abort triggered before auction started.');

promise_test(async test => {
  const uuid = generateUuid(test);

  // This doesn't have the header to be loaded in a fenced frame, but can still
  // check that it was requested, which is all this test needs.
  let trackingRenderURL =
      createTrackerURL(origin, uuid, `track_get`, `tracking_render_url`);

  await joinInterestGroup(
      test, uuid,
      {ads: [{renderURL: trackingRenderURL}]});

  let abortController = new AbortController();
  let fencedFrameConfig = await runBasicFledgeTestExpectingWinner(
      test, uuid, {signal: abortController.signal});

  // Aborting now should have no effect - in particular, it should still be
  // possible to navigate to the winning ad, and it should still send reports.
  abortController.abort('reason');

  // Load the fencedFrameConfig in a fenced frame, and make sure reports are
  // still sent and the render URL still loaded.
  createAndNavigateFencedFrame(test, fencedFrameConfig);
  await waitForObservedRequests(
      uuid,
      [trackingRenderURL, createBidderReportURL(uuid), createSellerReportURL(uuid)]);
}, 'Abort signalled after auction completes.');

promise_test(async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
    test, uuid,
    { biddingLogicURL: createBiddingScriptURL(
          { allowComponentAuction: true })});


  let abortController = new AbortController();
  let componentAuctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [window.location.origin],
    signal: abortController.signal
  };

  let auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig]
  };

  abortController.abort();
  // Aborting a component auction has no effect.
  await runBasicFledgeTestExpectingWinner(test, uuid, auctionConfigOverrides);
}, 'Abort component auction.');
