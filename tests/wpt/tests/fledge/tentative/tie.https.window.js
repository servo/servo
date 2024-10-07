// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long

"use strict;"

// Runs one auction at a time using `auctionConfigOverrides` until the auction
// has a winner.
async function runAuctionsUntilWinner(test, uuid, auctionConfigOverrides) {
  fencedFrameConfig = null;
  while (!fencedFrameConfig) {
    fencedFrameConfig =
        await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  }
  return fencedFrameConfig;
}

// This tests the case of ties. The winner of an auction is normally checked
// by these tests by checking a report sent when the winner is loaded in a fenced
// frame. Unfortunately, that requires a lot of navigations, which can be slow.
//
// So instead, run a multi-seller auction. The inner auction has two bidders,
// which both bid, and the seller gives them the same score. For the first
// auction, the top-level seller just accepts the only bid it sees, and then
// as usual, we navigate a fenced frame, to learn which bidder won.
//
// The for subsequent auctions, the nested component auction is identical,
// but the top-level auction rejects bids from the bidder that won the
// first auction. So if we have a winner, we know that the other bidder
// won the tie. Auctions are run in parallel until this happens.
//
// The interest groups use "group-by-origin" execution mode, to potentially
// allow the auctions run in parallel to complete faster.
promise_test(async test => {
  const uuid = generateUuid(test);

  // Use different report URLs for each interest group, to identify
  // which interest group has won an auction.
  let reportURLs = [createBidderReportURL(uuid, /*id=*/'1'),
                    createBidderReportURL(uuid, /*id=*/'2')];

  // Use different ad URLs for each auction. These need to be distinct
  // so that the top-level seller can check the URL to check if the
  // winning bid from the component auction has already won an
  // auction.
  let adURLs = [createRenderURL(uuid),
                createRenderURL(uuid, /*script=*/';')];

  await Promise.all(
      [ joinInterestGroup(
          test, uuid,
          { name: 'group 1',
            ads: [{ renderURL: adURLs[0] }],
            executionMode: 'group-by-origin',
            biddingLogicURL: createBiddingScriptURL(
                { allowComponentAuction: true,
                  reportWin: `sendReportTo("${reportURLs[0]}");`})}),
        joinInterestGroup(
          test, uuid,
          { name: 'group 2',
            ads: [{ renderURL: adURLs[1] }],
            executionMode: 'group-by-origin',
            biddingLogicURL: createBiddingScriptURL(
                { allowComponentAuction: true,
                  reportWin: `sendReportTo("${reportURLs[1]}");`})})
      ]
  );

  let componentAuctionConfig = {
      seller: window.location.origin,
      decisionLogicURL: createDecisionScriptURL(uuid),
      interestGroupBuyers: [window.location.origin]
  };

  let auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig]
  };

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);

  // Waiting for the report URL of the winner should succeed, while waiting for
  // the one of the loser should throw. Wait for both, see which succeeds, and
  // set "winningAdURL" to the ad URL of the winner.
  let winningAdURL = '';
  try {
    await waitForObservedRequests(uuid, [reportURLs[0]]);
    winningAdURL = adURLs[0];
  } catch (e) {
    await waitForObservedRequests(uuid, [reportURLs[1]]);
    winningAdURL = adURLs[1];
  }

  // Modify `auctionConfigOverrides` to only accept the ad from the interest
  // group that didn't win the first auction.
  auctionConfigOverrides.decisionLogicURL =
      createDecisionScriptURL(
        uuid,
        {scoreAd: `if (browserSignals.renderURL === "${winningAdURL}")
                     return 0;`});

  // Add an abort controller, so can cancel extra auctions.
  let abortController = new AbortController();
  auctionConfigOverrides.signal = abortController.signal;

  // Run a bunch of auctions in parallel, until one has a winner.
  let fencedFrameConfig = await Promise.any(
    [ runAuctionsUntilWinner(test, uuid, auctionConfigOverrides),
      runAuctionsUntilWinner(test, uuid, auctionConfigOverrides),
      runAuctionsUntilWinner(test, uuid, auctionConfigOverrides),
      runAuctionsUntilWinner(test, uuid, auctionConfigOverrides),
      runAuctionsUntilWinner(test, uuid, auctionConfigOverrides),
      runAuctionsUntilWinner(test, uuid, auctionConfigOverrides)
    ]
  );
  // Abort the other auctions.
  abortController.abort('reason');

  // Load the fencedFrameConfig in a fenced frame, and double-check that each
  // interest group has won once.
  createAndNavigateFencedFrame(test, fencedFrameConfig);
  await waitForObservedRequests(uuid, [reportURLs[0], reportURLs[1]]);
}, 'runAdAuction tie.');
