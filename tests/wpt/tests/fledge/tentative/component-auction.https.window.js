// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=/common/subset-tests.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-last

"use strict";

// Creates an AuctionConfig with a single component auction.
function createComponentAuctionConfig(uuid) {
  let componentAuctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [window.location.origin]
  };

  return {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig]
  };
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL()});

  await runBasicFledgeTestExpectingNoWinner(test, uuid, createComponentAuctionConfig(uuid));
}, 'Component auction allowed not specified by bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: false })});

  await runBasicFledgeTestExpectingNoWinner(test, uuid, createComponentAuctionConfig(uuid));
}, 'Component auction not allowed by bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true })});

  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.componentAuctions[0].decisionLogicURL = createDecisionScriptURL(
      uuid,
      { scoreAd: "return 5;" });

  await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfig);
}, 'Component auction allowed not specified by component seller.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true })});

  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.componentAuctions[0].decisionLogicURL = createDecisionScriptURL(
      uuid,
      { scoreAd: "return {desirability: 5, allowComponentAuction: false};" });

  await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfig);
}, 'Component auction not allowed by component seller.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true })});

  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.decisionLogicURL = createDecisionScriptURL(
      uuid,
      { scoreAd: "return 5;" });

  await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfig);
}, 'Component auction allowed not specified by top-level seller.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true })});

  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.interestGroupBuyers = [window.location.origin];

  try {
    await runBasicFledgeAuction(test, uuid, auctionConfig);
  } catch (exception) {
    assert_true(exception instanceof TypeError, "did not get expected error: " + exception);
    return;
  }
  throw 'Exception unexpectedly not thrown.'
}, 'Component auction top-level auction cannot have buyers.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true })});

  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.decisionLogicURL = createDecisionScriptURL(
      uuid,
      { scoreAd: "return {desirability: 5, allowComponentAuction: false};" });

  await runBasicFledgeTestExpectingNoWinner(test, uuid, auctionConfig);
}, 'Component auction not allowed by top-level seller.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Use distinct origins so can validate all origin parameters passed to worklets.
  let bidder = OTHER_ORIGIN1;
  let componentSeller = OTHER_ORIGIN2;
  let topLevelSeller = OTHER_ORIGIN3;

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/1);
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/2);

  // Note that generateBid() and reportWin() receive slightly different
  // "browserSignals" fields - only reportWin() gets "interestGroupOwner", so
  // need different sets of checks for them.
  await joinCrossOriginInterestGroup(
      test, uuid, bidder,
      { biddingLogicURL: createBiddingScriptURL(
        { origin: bidder,
          allowComponentAuction: true,
          generateBid:
            `if (browserSignals.seller !== "${componentSeller}")
               throw "Unexpected seller: " + browserSignals.seller;
             if (browserSignals.componentSeller !== undefined)
               throw "Unexpected componentSeller: " + browserSignals.componentSeller;
             if (browserSignals.topLevelSeller !== "${topLevelSeller}")
               throw "Unexpected topLevelSeller: " + browserSignals.topLevelSeller;
             if (browserSignals.interestGroupOwner !== undefined)
               throw "Unexpected interestGroupOwner: " + browserSignals.interestGroupOwner;
             if (browserSignals.topWindowHostname !== "${window.location.hostname}")
               throw "Unexpected topWindowHostname: " + browserSignals.topWindowHostname;`,
          reportWin:
            `if (browserSignals.seller !== "${componentSeller}")
               throw "Unexpected seller: " + browserSignals.seller;
             if (browserSignals.componentSeller !== undefined)
               throw "Unexpected componentSeller: " + browserSignals.componentSeller;
             if (browserSignals.topLevelSeller !== "${topLevelSeller}")
               throw "Unexpected topLevelSeller: " + browserSignals.topLevelSeller;
             if (browserSignals.interestGroupOwner !== "${bidder}")
               throw "Unexpected interestGroupOwner: " + browserSignals.interestGroupOwner;
             if (browserSignals.topWindowHostname !== "${window.location.hostname}")
               throw "Unexpected topWindowHostname: " + browserSignals.topWindowHostname;
             sendReportTo("${bidderReportURL}");`})});

  // Checks for scoreAd() and reportResult() for the component seller.
  let componentSellerChecks =
      `if (browserSignals.seller !== undefined)
         throw "Unexpected seller: " + browserSignals.seller;
       if (browserSignals.componentSeller !== undefined)
         throw "Unexpected componentSeller: " + browserSignals.componentSeller;
       if (browserSignals.topLevelSeller !== "${topLevelSeller}")
         throw "Unexpected topLevelSeller: " + browserSignals.topLevelSeller;
       if (browserSignals.interestGroupOwner !== "${bidder}")
         throw "Unexpected interestGroupOwner: " + browserSignals.interestGroupOwner;
       if (browserSignals.topWindowHostname !== "${window.location.hostname}")
         throw "Unexpected topWindowHostname: " + browserSignals.topWindowHostname;`;

  let componentAuctionConfig = {
    seller: componentSeller,
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { origin: componentSeller,
          scoreAd: componentSellerChecks,
          reportResult: `${componentSellerChecks}
                          sendReportTo("${componentSellerReportURL}");` }),
    interestGroupBuyers: [bidder]
  };

  // Checks for scoreAd() and reportResult() for the top-level seller.
  let topLevelSellerChecks =
      `if (browserSignals.seller !== undefined)
         throw "Unexpected seller: " + browserSignals.seller;
       if (browserSignals.componentSeller !== "${componentSeller}")
         throw "Unexpected componentSeller: " + browserSignals.componentSeller;
       if (browserSignals.topLevelSeller !== undefined)
         throw "Unexpected topLevelSeller: " + browserSignals.topLevelSeller;
       if (browserSignals.interestGroupOwner !== "${bidder}")
         throw "Unexpected interestGroupOwner: " + browserSignals.interestGroupOwner;
       if (browserSignals.topWindowHostname !== "${window.location.hostname}")
         throw "Unexpected topWindowHostname: " + browserSignals.topWindowHostname;`;

  let auctionConfigOverrides = {
    seller: topLevelSeller,
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { origin: topLevelSeller,
          scoreAd: topLevelSellerChecks,
          reportResult: `${topLevelSellerChecks}
                         sendReportTo("${topLevelSellerReportURL}");` }),
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig]
  };

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Component auction browserSignals origins.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/1);
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/2);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true,
          bid: 5,
          reportWin:
            `if (browserSignals.bid !== 5)
               throw "Unexpected bid: " + browserSignals.bid;
             sendReportTo("${bidderReportURL}");`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
    createDecisionScriptURL(
        uuid,
        { scoreAd:
              `if (bid !== 5)
                 throw "Unexpected component bid: " + bid`,
          reportResult:
              `if (browserSignals.bid !== 5)
                 throw "Unexpected component bid: " + browserSignals.bid;
               if (browserSignals.modifiedBid !== undefined)
                 throw "Unexpected component modifiedBid: " + browserSignals.modifiedBid;
               sendReportTo("${componentSellerReportURL}");` });

  auctionConfig.decisionLogicURL =
    createDecisionScriptURL(
        uuid,
        { scoreAd:
              `if (bid !== 5)
                 throw "Unexpected top-level bid: " + bid`,
          reportResult:
              `if (browserSignals.bid !== 5)
                 throw "Unexpected top-level bid: " + browserSignals.bid;
               if (browserSignals.modifiedBid !== undefined)
                 throw "Unexpected top-level modifiedBid: " + browserSignals.modifiedBid;
               sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Component auction unmodified bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/1);
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/2);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true,
          bid: 5,
          reportWin:
            `if (browserSignals.bid !== 5)
               throw "Unexpected bid: " + browserSignals.bid;
             sendReportTo("${bidderReportURL}");`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 5)
                   throw "Unexpected component bid: " + bid
                 return {desirability: 5, allowComponentAuction: true, bid: 4};`,
            reportResult:
                `if (browserSignals.bid !== 5)
                   throw "Unexpected component bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== 4)
                   throw "Unexpected component modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${componentSellerReportURL}");` });

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 4)
                   throw "Unexpected top-level bid: " + bid`,
            reportResult:
                `if (browserSignals.bid !== 4)
                   throw "Unexpected top-level bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected top-level modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Component auction modified bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/1);
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/2);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true,
          bid: 5,
          reportWin:
            `if (browserSignals.bid !== 5)
               throw "Unexpected bid: " + browserSignals.bid;
             sendReportTo("${bidderReportURL}");`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 5)
                   throw "Unexpected component bid: " + bid
                 return {desirability: 5, allowComponentAuction: true, bid: 5};`,
            reportResult:
                `if (browserSignals.bid !== 5)
                   throw "Unexpected component bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== 5)
                   throw "Unexpected component modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${componentSellerReportURL}");` });

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 5)
                   throw "Unexpected top-level bid: " + bid`,
            reportResult:
                `if (browserSignals.bid !== 5)
                   throw "Unexpected top-level bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected top-level modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Component auction modified bid to same value.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/1);
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/2);

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true,
          bid: 5,
          reportWin:
            `if (browserSignals.bid !== 5)
               throw "Unexpected bid: " + browserSignals.bid;
             sendReportTo("${bidderReportURL}");`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 5)
                   throw "Unexpected component bid: " + bid`,
            reportResult:
                `if (browserSignals.bid !== 5)
                   throw "Unexpected component bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected component modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${componentSellerReportURL}");` });

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `if (bid !== 5)
                   throw "Unexpected top-level bid: " + bid
                 return {desirability: 5, allowComponentAuction: true, bid: 4};`,
            reportResult:
                `if (browserSignals.bid !== 5)
                   throw "Unexpected top-level bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected top-level modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Top-level auction cannot modify bid.');
