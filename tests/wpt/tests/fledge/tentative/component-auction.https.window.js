// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/subset-tests.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last

"use strict";

// Creates an AuctionConfig with a single component auction.
function createComponentAuctionConfig(uuid, auctionConfigOverrides = {},
                                      deprecatedRenderURLReplacements = {}) {
  let componentAuctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [window.location.origin],
    deprecatedRenderURLReplacements: deprecatedRenderURLReplacements
  };

  return {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig],
    ...auctionConfigOverrides
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
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

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
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

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
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

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
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

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
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderReportURL = createBidderReportURL(uuid);
  let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

  await joinInterestGroup(
      test, uuid,
      { biddingLogicURL: createBiddingScriptURL(
        { allowComponentAuction: true,
          reportWin:
            `if (browserSignals.desirability !== undefined)
               throw "Unexpected desirability: " + browserSignals.desirability;
             sendReportTo("${bidderReportURL}");`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 3, allowComponentAuction: true};`,
            reportResult:
                `if (browserSignals.desirability !== 3)
                  throw "Unexpected component desirability: " + browserSignals.desirability;
                 sendReportTo("${componentSellerReportURL}");` });

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 4, allowComponentAuction: true};`,
            reportResult:
                `if (browserSignals.desirability !== 4)
                  throw "Unexpected component desirability: " + browserSignals.desirability;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL]);
}, 'Component auction desirability.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // An auction with two components, each of which has a distinct bidder origin,
  // so the bidder in the second component is OTHER_ORIGIN1). The bidder in the
  // first component auction bids more and is given the highest of all
  // desirability scores in the auction by its component seller, but the
  // top-level seller prefers bidder 2.
  let bidder1ReportURL = createBidderReportURL(uuid, /*id=*/1);
  let bidder2ReportURL = createBidderReportURL(uuid, /*id=*/2);
  let componentSeller1ReportURL = createSellerReportURL(uuid, /*id=*/"component1");
  let componentSeller2ReportURL = createSellerReportURL(uuid, /*id=*/"component2");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

  await Promise.all([
      joinInterestGroup(
          test, uuid,
          { biddingLogicURL: createBiddingScriptURL(
            { bid: 10,
              allowComponentAuction: true,
              reportWin:
                `sendReportTo("${bidder1ReportURL}");`})}),
      joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1,
        { biddingLogicURL: createBiddingScriptURL(
          { origin: OTHER_ORIGIN1,
            bid: 2,
            allowComponentAuction: true,
            reportWin:
              `if (browserSignals.bid !== 2)
                 throw "Unexpected bid: " + browserSignals.bid;
               sendReportTo("${bidder2ReportURL}");`})})
  ]);

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 10, allowComponentAuction: true};`,
            reportResult:
                `sendReportTo("${componentSeller1ReportURL}");` });

  auctionConfig.componentAuctions[1] = {
    ...auctionConfig.componentAuctions[0],
    interestGroupBuyers: [OTHER_ORIGIN1],
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd:
              `return {desirability: 1, allowComponentAuction: true};`,
          reportResult:
              `if (browserSignals.desirability !== 1)
                 throw "Unexpected component desirability: " + browserSignals.desirability;
               sendReportTo("${componentSeller2ReportURL}");` })
  }

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 11 - bid, allowComponentAuction: true};`,
            reportResult:
                `if (browserSignals.desirability !== 9)
                   throw "Unexpected component desirability: " + browserSignals.desirability;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidder2ReportURL, componentSeller2ReportURL, topLevelSellerReportURL]);
}, 'Component auction desirability two sellers, two bidders.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let renderURL1 = createRenderURL(uuid);
  let renderURL2 = createRenderURL(uuid, /*script=*/';');

  // The same bidder uses different ads, bids, and reporting URLs for different
  // component sellers.
  let bidderReportURL1 = createBidderReportURL(uuid, /*id=*/1);
  let bidderReportURL2 = createBidderReportURL(uuid, /*id=*/2);
  let componentSeller1ReportURL = createSellerReportURL(uuid, /*id=*/"component1");
  let componentSeller2ReportURL = createSellerReportURL(uuid, /*id=*/"component2");
  let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

  await joinInterestGroup(
        test, uuid,
        { ads: [{ renderURL: renderURL1 }, { renderURL: renderURL2 }],
          biddingLogicURL: createBiddingScriptURL(
          { allowComponentAuction: true,
            generateBid:
              // "auctionSignals" contains the bid and the report URL, to
              // make the same bidder behave differently in the two
              // auctions.
              'return auctionSignals;',
            reportWin:
              `if (browserSignals.renderURL !== "${renderURL2}")
                 throw "Wrong winner: " + browserSignals.renderURL;
               sendReportTo(auctionSignals.reportURL);`})});

  let auctionConfig = createComponentAuctionConfig(uuid);

  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 10, allowComponentAuction: true};`,
            reportResult:
                `sendReportTo("${componentSeller1ReportURL}");` });
  // "auctionSignals" contains the bid and the report URL, to
  // make the same bidder behave differently in the two
  // auctions.
  auctionConfig.componentAuctions[0].auctionSignals = {
    bid: 10,
    allowComponentAuction: true,
    render: renderURL1,
    reportURL: bidderReportURL1
  };

  auctionConfig.componentAuctions[1] = {
    ...auctionConfig.componentAuctions[0],
    auctionSignals: {
      bid: 2,
      allowComponentAuction: true,
      render: renderURL2,
      reportURL: bidderReportURL2
    },
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd:
              `return {desirability: 1, allowComponentAuction: true};`,
          reportResult:
              `if (browserSignals.desirability !== 1)
                 throw "Unexpected component desirability: " + browserSignals.desirability;
               if (browserSignals.renderURL !== "${renderURL2}")
                 throw "Wrong winner: " + browserSignals.renderURL;
               sendReportTo("${componentSeller2ReportURL}");` })
  }

  auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd:
                `return {desirability: 11 - bid, allowComponentAuction: true};`,
            reportResult:
                `if (browserSignals.desirability !== 9)
                   throw "Unexpected component desirability: " + browserSignals.desirability;
                 if (browserSignals.renderURL !== "${renderURL2}")
                   throw "Wrong winner: " + browserSignals.renderURL;
                 sendReportTo("${topLevelSellerReportURL}");` });

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL2, componentSeller2ReportURL, topLevelSellerReportURL]);
}, 'Component auction desirability and renderURL two sellers, one bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // The renderURLs / report URLs for the first/second iterations of the auction.
  let renderURL1 = createRenderURL(uuid);
  let renderURL2 = createRenderURL(uuid, /*script=*/';');
  let bidderReportURL1 = createBidderReportURL(uuid, /*id=*/1);
  let bidderReportURL2 = createBidderReportURL(uuid, /*id=*/2);
  let seller1ReportURL = createSellerReportURL(uuid, /*id=*/1);
  let seller2ReportURL = createSellerReportURL(uuid, /*id=*/2);

  await joinInterestGroup(
        test, uuid,
        { ads: [{ renderURL: renderURL1 }, { renderURL: renderURL2 }],
          biddingLogicURL: createBiddingScriptURL(
          { allowComponentAuction: true,
            generateBid:
              `// If this is the first recorded win, use "renderURL1"
               if (browserSignals.bidCount === 0 &&
                   browserSignals.prevWinsMs.length === 0) {
                 return {bid: 2, allowComponentAuction: true, render: "${renderURL1}"};
               }

               // Otherwise, check that a single bid and win were reported, despite the
               // bidder bidding twice in the first auction, once for each component
               // auction.
               if (browserSignals.bidCount === 1 &&
                   browserSignals.prevWinsMs.length === 1 &&
                   typeof browserSignals.prevWinsMs[0][0] === "number" &&
                   browserSignals.prevWinsMs[0][1].renderURL === "${renderURL1}") {
                 return {bid: 1, allowComponentAuction: true, render: "${renderURL2}"};
               }
               throw "Unexpected browserSignals: " + JSON.stringify(browserSignals);`,
            reportWin:
              `if (browserSignals.renderURL === "${renderURL1}")
                 sendReportTo("${bidderReportURL1}");
               if (browserSignals.renderURL === "${renderURL2}")
                 sendReportTo("${bidderReportURL2}");`})});

  // Auction has two component auctions with different sellers but the same
  // single bidder. The first component auction only accepts bids with
  // "renderURL1", the second only accepts bids with "renderURL2".
  let auctionConfig = createComponentAuctionConfig(uuid);
  auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
          uuid,
          { scoreAd: `if (browserSignals.renderURL !== '${renderURL1}')
                        throw 'Wrong ad';`,
            reportResult: `sendReportTo('${seller1ReportURL}');`}
      );

  auctionConfig.componentAuctions[1] = {
      seller: OTHER_ORIGIN1,
      interestGroupBuyers: [window.location.origin],
      decisionLogicURL: createDecisionScriptURL(
          uuid,
          { origin: OTHER_ORIGIN1,
            scoreAd: `if (browserSignals.renderURL !== '${renderURL2}')
                        throw 'Wrong ad';`,
            reportResult: `sendReportTo('${seller2ReportURL}');`}
      )
  };

  // In the first auction, the bidder should use "renderURL1", which the first
  // component auction allows. `prevWinsMs` and `numBids` should be updated.
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL1, seller1ReportURL]);

  // In the second auction, the bidder should use "renderURL2", which the second
  // component auction allows. `prevWinsMs` and `numBids` should reflect the updated
  // value.
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
  await waitForObservedRequests(
      uuid,
      [bidderReportURL1, seller1ReportURL, bidderReportURL2, seller2ReportURL]);
}, `Component auction prevWinsMs and numBids updating in one component seller's auction, read in another's.`);


const makeDeprecatedRenderURLReplacementTest = ({
  name,
  deprecatedRenderURLReplacements,
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);

    let bidderReportURL = createBidderReportURL(uuid);
    let componentSellerReportURL = createSellerReportURL(uuid, /*id=*/"component");
    let topLevelSellerReportURL = createSellerReportURL(uuid, /*id=*/"top");

    // These are used within the URLs for deprecatedRenderURLReplacement tests.
    const renderURLReplacementsStrings = createStringBeforeAndAfterReplacements(deprecatedRenderURLReplacements);
    const beforeReplacementsString = renderURLReplacementsStrings.beforeReplacements;
    const afterReplacementsString = renderURLReplacementsStrings.afterReplacements;
    const renderURLBeforeReplacements = createTrackerURL(window.location.origin, uuid, 'track_get', beforeReplacementsString);
    const renderURLAfterReplacements = createTrackerURL(window.location.origin, uuid, 'track_get', afterReplacementsString);

    await joinInterestGroup(
      test, uuid,
      {
        ads: [{ renderURL: renderURLBeforeReplacements }],
        biddingLogicURL: createBiddingScriptURL(
          {
            allowComponentAuction: true,
            bid: 5,
            reportWin:
              `if (browserSignals.bid !== 5)
                 throw "Unexpected bid: " + browserSignals.bid;
               sendReportTo("${bidderReportURL}");`
          })
      });

    let auctionConfig = createComponentAuctionConfig(uuid, {}, deprecatedRenderURLReplacements);

    auctionConfig.componentAuctions[0].decisionLogicURL =
      createDecisionScriptURL(
        uuid,
        {
          scoreAd:
            `if (bid !== 5)
                   throw "Unexpected component bid: " + bid`,
          reportResult:
            `if (browserSignals.bid !== 5)
                   throw "Unexpected component bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected component modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${componentSellerReportURL}");`
        });

    auctionConfig.decisionLogicURL =
      createDecisionScriptURL(
        uuid,
        {
          scoreAd:
            `if (bid !== 5)
                   throw "Unexpected top-level bid: " + bid`,
          reportResult:
            `if (browserSignals.bid !== 5)
                   throw "Unexpected top-level bid: " + browserSignals.bid;
                 if (browserSignals.modifiedBid !== undefined)
                   throw "Unexpected top-level modifiedBid: " + browserSignals.modifiedBid;
                 sendReportTo("${topLevelSellerReportURL}");`
        });

    await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfig);
    await waitForObservedRequests(
      uuid,
      [bidderReportURL, componentSellerReportURL, topLevelSellerReportURL, renderURLAfterReplacements]);
  }, name);
};

makeDeprecatedRenderURLReplacementTest({
  name: 'Replacements with brackets.',
  deprecatedRenderURLReplacements: { '${EXAMPLE-MACRO}': 'SSP' }
});

makeDeprecatedRenderURLReplacementTest({
  name: 'Replacements with percents.',
  deprecatedRenderURLReplacements: { '%%EXAMPLE-MACRO%%': 'SSP' }
});

makeDeprecatedRenderURLReplacementTest({
  name: 'Replacements with multiple replacements.',
  deprecatedRenderURLReplacements: { '${EXAMPLE-MACRO1}': 'SSP1', '%%EXAMPLE-MACRO2%%': 'SSP2' }
});

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let deprecatedRenderURLReplacements = { '${EXAMPLE-MACRO1}': 'SSP1', '%%EXAMPLE-MACRO2%%': 'SSP2' };
  const renderURLReplacementsStrings = createStringBeforeAndAfterReplacements(deprecatedRenderURLReplacements);
  let beforeReplacementsString = renderURLReplacementsStrings.beforeReplacements;

  await joinInterestGroup(
    test, uuid,
    {
      ads: [{ renderURL: createTrackerURL(window.location.origin, uuid, 'track_get', beforeReplacementsString) }],
      biddingLogicURL: createBiddingScriptURL({allowComponentAuction: true})
    });
  let auctionConfigOverride = {deprecatedRenderURLReplacements: deprecatedRenderURLReplacements }
  let auctionConfig = createComponentAuctionConfig(uuid,/*auctionConfigOverride=*/auctionConfigOverride,
  /*deprecatedRenderURLReplacements=*/deprecatedRenderURLReplacements);

  auctionConfig.componentAuctions[0].decisionLogicURL = createDecisionScriptURL(uuid);

    try {
      await runBasicFledgeAuction(test, uuid, auctionConfig);
    } catch (exception) {
      assert_true(exception instanceof TypeError, "did not get expected error: " + exception);
      return;
    }
    throw 'Exception unexpectedly not thrown.'
}, "deprecatedRenderURLReplacements cause error if passed in top level auction and component auction.");
