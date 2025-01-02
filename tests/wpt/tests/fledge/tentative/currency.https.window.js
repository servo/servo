// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-8
// META: variant=?9-12
// META: variant=?13-16
// META: variant=?17-20
// META: variant=?21-24
// META: variant=?25-28
// META: variant=?29-32
// META: variant=?33-last

'use strict;'

const ORIGIN = window.location.origin;

// The tests in this file focus on calls to runAdAuction involving currency
// handling.

// Joins an interest group that bids 9USD on window.location.origin, and one
// that bids 10CAD on OTHER_ORIGIN1, each with a reportWin() report.
async function joinTwoCurrencyGroups(test, uuid) {
  const reportWinURL = createBidderReportURL(uuid, 'USD');
  const biddingURL = createBiddingScriptURL(
      {bidCurrency: 'USD', reportWin: `sendReportTo('${reportWinURL}')`});
  await joinInterestGroup(test, uuid, {biddingLogicURL: biddingURL});

  const otherReportWinURL = createBidderReportURL(uuid, 'CAD', OTHER_ORIGIN1);
  const otherBiddingURL = createBiddingScriptURL({
    origin: OTHER_ORIGIN1,
    bid: 10,
    bidCurrency: 'CAD',
    reportWin: `sendReportTo('${otherReportWinURL}')`
  });
  await joinCrossOriginInterestGroup(
      test, uuid, OTHER_ORIGIN1, {biddingLogicURL: otherBiddingURL});
}

function createBiddingScriptURLWithCurrency(uuid, currency) {
  return createBiddingScriptURL({
    bidCurrency: currency,
    allowComponentAuction: true,
    reportWin: `
        sendReportTo('${createBidderReportURL(uuid, /*id=*/ '')}' +
                     browserSignals.bid + browserSignals.bidCurrency);`,
  });
}

// Creates a component-auction eligible bidding script returning a bid `bid` in
// currency `currency`. It provides a reporting handler that logs bid and
// highestScoringOtherBid along with their currencies.
function createBiddingScriptURLForHighestScoringOther(uuid, bid, currency) {
  return createBiddingScriptURL({
    bid: bid,
    bidCurrency: currency,
    allowComponentAuction: true,
    generateBid: `
      forDebuggingOnly.reportAdAuctionWin(
          '${createBidderReportURL(uuid, /*id=*/ 'dbg_')}' +
          '\${winningBid}\${winningBidCurrency}_' +
          '\${highestScoringOtherBid}\${highestScoringOtherBidCurrency}');`,
    reportWin: `
        sendReportTo(
            '${createBidderReportURL(uuid, /*id=*/ '')}' +
            browserSignals.bid + browserSignals.bidCurrency +
            '_' + browserSignals.highestScoringOtherBid +
            browserSignals.highestScoringOtherBidCurrency);`,
  });
}

function createDecisionURLExpectCurrency(uuid, currencyInScore) {
  return createDecisionScriptURL(uuid, {
    scoreAd: `
            if (browserSignals.bidCurrency !== '${currencyInScore}')
              throw 'Wrong currency';`,
    reportResult: `
          sendReportTo('${createSellerReportURL(uuid, /*id=*/ '')}' +
                         browserSignals.bid + browserSignals.bidCurrency);`,
  });
}

// Creates a component-auction seller script, which by default just scores
// bid * 2, but the `conversion` argument can be used to customize bid
// modification and currenct conversion.
//
// The script provides a reporting handler that logs bid and
// highestScoringOtherBid along with their currencies as well as `suffix`.
function createDecisionURLForHighestScoringOther(
    uuid, conversion = '', suffix = '') {
  return createDecisionScriptURL(uuid, {
    scoreAd: `
      forDebuggingOnly.reportAdAuctionWin(
          '${createSellerReportURL(uuid, /*id=*/ 'dbg_')}' + '${suffix}' +
          '\${winningBid}\${winningBidCurrency}_' +
          '\${highestScoringOtherBid}\${highestScoringOtherBidCurrency}');
      let converted = undefined;
      let modified = undefined;
      let modifiedCurrency = undefined;
      ${conversion}
      return {desirability: 2 * bid,
              incomingBidInSellerCurrency: converted,
              bid: modified,
              bidCurrency: modifiedCurrency,
              allowComponentAuction: true};
    `,
    reportResult: `
        sendReportTo(
            '${createSellerReportURL(uuid, /*id=*/ '')}' + '${suffix}' +
            browserSignals.bid + browserSignals.bidCurrency +
            '_' + browserSignals.highestScoringOtherBid +
            browserSignals.highestScoringOtherBidCurrency);`,
  });
}

// Joins groups for 9USD and 10USD, with reporting including
// highestScoringOtherBid.
async function joinTwoGroupsForHighestScoringOther(test, uuid) {
  await joinInterestGroup(test, uuid, {
    name: 'group-9USD',
    biddingLogicURL:
        createBiddingScriptURLForHighestScoringOther(uuid, /*bid=*/ 9, 'USD')
  });
  await joinInterestGroup(test, uuid, {
    name: 'group-10USD',
    biddingLogicURL:
        createBiddingScriptURLForHighestScoringOther(uuid, /*bid=*/ 10, 'USD')
  });
}

async function runCurrencyComponentAuction(test, uuid, params = {}) {
  let auctionConfigOverrides = {
    interestGroupBuyers: [],
    decisionLogicURL: createDecisionScriptURL(uuid, {
      reportResult: `
        sendReportTo('${createSellerReportURL(uuid, 'top_')}' +
                     browserSignals.bid + browserSignals.bidCurrency)`,
      ...params.topLevelSellerScriptParamsOverride
    }),
    componentAuctions: [{
      seller: ORIGIN,
      decisionLogicURL: createDecisionScriptURL(uuid, {
        reportResult: `
          sendReportTo('${createSellerReportURL(uuid, 'component_')}' +
                       browserSignals.bid + browserSignals.bidCurrency)`,
        ...params.componentSellerScriptParamsOverride
      }),
      interestGroupBuyers: [ORIGIN],
      ...params.componentAuctionConfigOverrides
    }],
    ...params.topLevelAuctionConfigOverrides
  };
  return await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
}

// Runs a component auction with reporting scripts that report bid and
// highestScoringOtherBid, along with their currencies.
//
// Customization points in `params` are:
// componentAuctionConfigOverrides, topLevelAuctionConfigOverrides:
// edit auctionConfig for given auction level.
//
// topLevelConversion and componentConversion:
// Permit customizing how the scoring function does currency conversiona and
// bid modification. See createDecisionURLForHighestScoringOther().
async function runCurrencyComponentAuctionForHighestScoringOther(
    test, uuid, params = {}) {
  let auctionConfigOverrides = {
    interestGroupBuyers: [],
    decisionLogicURL: createDecisionURLForHighestScoringOther(
        uuid, params.topLevelConversion || '', 'top_'),
    componentAuctions: [{
      seller: ORIGIN,
      decisionLogicURL: createDecisionURLForHighestScoringOther(
          uuid, params.componentConversion || '', 'component_'),
      interestGroupBuyers: [ORIGIN],
      ...params.componentAuctionConfigOverrides
    }],
    ...params.topLevelAuctionConfigOverrides
  };
  return await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURL({bidCurrency: 'usd'})});
  await runBasicFledgeTestExpectingNoWinner(test, uuid);
}, 'Returning bid with invalid currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      {decisionLogicURL: createDecisionURLExpectCurrency(uuid, 'USD')});
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '9???'), createBidderReportURL(uuid, '9???')
  ]);
}, 'Returning bid with currency, configuration w/o currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, undefined)});
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    perBuyerCurrencies: {'*': 'USD'},
    decisionLogicURL: createDecisionURLExpectCurrency(uuid, '???')
  });
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '9USD'), createBidderReportURL(uuid, '9USD')
  ]);
}, 'Returning bid w/o currency, configuration w/currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    perBuyerCurrencies: {'*': 'USD'},
    decisionLogicURL: createDecisionURLExpectCurrency(uuid, 'USD')
  });
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '9USD'), createBidderReportURL(uuid, '9USD')
  ]);
}, 'Returning bid w/currency, configuration w/matching currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURL({bidCurrency: 'USD'})});
  await runBasicFledgeTestExpectingNoWinner(
      test, uuid, {perBuyerCurrencies: {'*': 'CAD'}});
}, 'Returning bid w/currency, configuration w/different currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoCurrencyGroups(test, uuid);
  let auctionConfigOverrides = {
    interestGroupBuyers: [ORIGIN, OTHER_ORIGIN1],
    perBuyerCurrencies: {}
  };
  auctionConfigOverrides.perBuyerCurrencies['*'] = 'USD';
  auctionConfigOverrides.perBuyerCurrencies[OTHER_ORIGIN1] = 'CAD';
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);

  // Since the scoring script doesn't actually look at the currencies,
  // We expect 10CAD to win because 10 > 9
  await waitForObservedRequests(uuid, [
    createBidderReportURL(uuid, 'CAD', OTHER_ORIGIN1),
    createSellerReportURL(uuid)
  ]);
}, 'Different currencies for different origins, all match.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoCurrencyGroups(test, uuid);
  let auctionConfigOverrides = {
    interestGroupBuyers: [ORIGIN, OTHER_ORIGIN1],
    perBuyerCurrencies: {}
  };
  auctionConfigOverrides.perBuyerCurrencies[ORIGIN] = 'USD';
  auctionConfigOverrides.perBuyerCurrencies[OTHER_ORIGIN1] = 'EUR';
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);

  // Since the configuration for CAD script expects EUR only the USD bid goes
  // through.
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid, 'USD'), createSellerReportURL(uuid)]);
}, 'Different currencies for different origins, USD one matches.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoCurrencyGroups(test, uuid);
  let auctionConfigOverrides = {
    interestGroupBuyers: [ORIGIN, OTHER_ORIGIN1],
    perBuyerCurrencies: {}
  };
  auctionConfigOverrides.perBuyerCurrencies['*'] = 'EUR';
}, 'Different currencies for different origins, none match.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
              if (browserSignals.bidCurrency !== 'USD')
                throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  // While scoring sees the original currency tag, reporting currency tags are
  // config-based.
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_9???'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- no currency restriction.');


subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'USD'},
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                if (browserSignals.bidCurrency !== 'USD')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  // Because component's sellerCurrency is USD, the bid it makes is seen to be
  // in dollars by top-level reporting. That doesn't affect reporting in its
  // own auction.
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_9USD'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- component sellerCurrency matches bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'EUR'},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid, bidCurrency: 'EUR'}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                if (browserSignals.bidCurrency !== 'EUR')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  // Because component's sellerCurrency is USD, the bid it makes is seen to be
  // in dollars by top-level reporting. That doesn't affect reporting in its
  // own auction.
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_13.5EUR'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- component scoreAd modifies bid into its sellerCurrency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'EUR'},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                // scoreAd sees what's actually passed in.
                if (browserSignals.bidCurrency !== '???')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_13.5EUR'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- component scoreAd modifies bid, no explicit currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides:
        {sellerCurrency: 'EUR', perBuyerCurrencies: {'*': 'USD'}},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                // scoreAd sees what's actually passed in.
                if (browserSignals.bidCurrency !== '???')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_13.5EUR'),
    createSellerReportURL(uuid, 'component_9USD'),
    createBidderReportURL(uuid, '9USD')
  ]);
}, 'Multi-seller auction --- component scoreAd modifies bid, bidder has bidCurrency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {perBuyerCurrencies: {'*': 'USD'}},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                // scoreAd sees what's actually passed in.
                if (browserSignals.bidCurrency !== '???')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_13.5???'),
    createSellerReportURL(uuid, 'component_9USD'),
    createBidderReportURL(uuid, '9USD')
  ]);
}, 'Multi-seller auction --- only bidder currency specified.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {perBuyerCurrencies: {'*': 'USD'}},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid, bidCurrency: 'CAD'}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                // scoreAd sees what's actually passed in.
                if (browserSignals.bidCurrency !== 'CAD')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_13.5???'),
    createSellerReportURL(uuid, 'component_9USD'),
    createBidderReportURL(uuid, '9USD')
  ]);
}, 'Multi-seller auction --- only bidder currency in config, component uses explicit currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, {
    biddingLogicURL:
        createBiddingScriptURLWithCurrency(uuid, /*bidCurrency=*/ undefined)
  });
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'CAD'},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 incomingBidInSellerCurrency: 12345}
      `
    },
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
                // scoreAd sees what's actually passed in.
                if (bid !== 9)
                  throw 'Wrong bid';
                if (browserSignals.bidCurrency !== '???')
                  throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_9CAD'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- incomingBidInSellerCurrency does not go to top-level; component sellerCurrency does.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let result = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'EUR'},
    componentSellerScriptParamsOverride: {
      scoreAd: `
        return {desirability: 2 * bid, allowComponentAuction: true,
                 bid: 1.5 * bid, bidCurrency: 'CAD'}
      `
    }
  });
  expectNoWinner(result);
}, 'Multi-seller auction --- component scoreAd modifies bid to wrong currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let topLevelConfigOverride = {perBuyerCurrencies: {}};
  topLevelConfigOverride.perBuyerCurrencies[ORIGIN] = 'USD';
  let config = await runCurrencyComponentAuction(test, uuid, {
    topLevelAuctionConfigOverrides: topLevelConfigOverride,
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
              if (browserSignals.bidCurrency !== 'USD')
                throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  // Because component is constrained by perBuyerCurrencies for it on top-level
  // to USD, the bid it makes is seen to be in dollars by top-level reporting.
  // That doesn't affect reporting in its own auction.
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_9USD'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- top-level perBuyerCurrencies matches bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let topLevelConfigOverride = {perBuyerCurrencies: {}};
  topLevelConfigOverride.perBuyerCurrencies[ORIGIN] = 'USD';
  let config = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'USD'},
    topLevelAuctionConfigOverrides: topLevelConfigOverride,
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
              if (browserSignals.bidCurrency !== 'USD')
                throw 'Wrong currency';`
    }
  });
  expectSuccess(config);
  createAndNavigateFencedFrame(test, config);
  // Because component is constrained by perBuyerCurrencies for it on top-level
  // to USD, the bid it makes is seen to be in dollars by top-level reporting.
  // That doesn't affect reporting in its own auction.
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_9USD'),
    createSellerReportURL(uuid, 'component_9???'),
    createBidderReportURL(uuid, '9???')
  ]);
}, 'Multi-seller auction --- consistent sellerConfig and top-level perBuyerCurrencies.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let topLevelConfigOverride = {perBuyerCurrencies: {}};
  topLevelConfigOverride.perBuyerCurrencies[ORIGIN] = 'EUR';
  let result = await runCurrencyComponentAuction(test, uuid, {
    componentAuctionConfigOverrides: {sellerCurrency: 'USD'},
    topLevelAuctionConfigOverrides: topLevelConfigOverride,
    topLevelSellerScriptParamsOverride: {
      scoreAd: `
              if (browserSignals.bidCurrency !== 'USD')
                throw 'Wrong currency';`
    }
  });
  expectNoWinner(result);
}, 'Multi-seller auction --- inconsistent sellerConfig and top-level perBuyerCurrencies.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let topLevelConfigOverride = {perBuyerCurrencies: {}};
  topLevelConfigOverride.perBuyerCurrencies[ORIGIN] = 'EUR';

  let result = await runCurrencyComponentAuction(
      test, uuid, {componentAuctionConfigOverrides: topLevelConfigOverride});
  expectNoWinner(result);
}, 'Multi-seller auction --- top-level perBuyerCurrencies different from bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  let result = await runCurrencyComponentAuction(
      test, uuid, {componentAuctionConfigOverrides: {sellerCurrency: 'EUR'}});
  expectNoWinner(result);
}, 'Multi-seller auction --- component sellerCurrency different from bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);
  await runBasicFledgeTestExpectingNoWinner(test, uuid, {
    decisionLogicURL: createDecisionScriptURL(uuid, {
      scoreAd: `
          return {desirability: 2 * bid,
                  incomingBidInSellerCurrency: 5* bid}
        `
    })
  });
}, 'Trying to use incomingBidInSellerCurrency w/o sellerCurrency set.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);
  await runBasicFledgeTestExpectingWinner(test, uuid, {
    decisionLogicURL: createDecisionScriptURL(uuid, {
      scoreAd: `
          return {desirability: 2 * bid,
                  incomingBidInSellerCurrency: 5* bid}
        `,
    }),
    sellerCurrency: 'USD'
  });
}, 'Trying to use incomingBidInSellerCurrency w/sellerCurrency set.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  await runBasicFledgeTestExpectingNoWinner(test, uuid, {
    decisionLogicURL: createDecisionScriptURL(uuid, {
      scoreAd: `
          return {desirability: 2 * bid,
                  incomingBidInSellerCurrency: 5* bid}
        `
    }),
    sellerCurrency: 'USD'
  });
}, 'Trying to use incomingBidInSellerCurrency to change bid already in that currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLWithCurrency(uuid, 'USD')});
  await runBasicFledgeTestExpectingWinner(test, uuid, {
    decisionLogicURL: createDecisionScriptURL(uuid, {
      scoreAd: `
          return {desirability: 2 * bid,
                  incomingBidInSellerCurrency: bid}
        `
    }),
    sellerCurrency: 'USD'
  });
}, 'incomingBidInSellerCurrency repeating value of bid already in that currency is OK.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      {decisionLogicURL: createDecisionURLForHighestScoringOther(uuid)});
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '10???_9???'),
    createBidderReportURL(uuid, '10???_9???'),
    // w/o sellerCurrency set, forDebuggingOnly reports original values and ???
    // as tags.
    createSellerReportURL(uuid, 'dbg_10???_9???'),
    createBidderReportURL(uuid, 'dbg_10???_9???')
  ]);
}, 'Converted currency use with no sellerCurrency set.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionURLForHighestScoringOther(uuid),
    sellerCurrency: 'USD'
  });
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '10???_9USD'),
    createBidderReportURL(uuid, '10???_9USD'),
    // w/sellerCurrency set, forDebuggingOnly reports converted bids +
    // sellerCurrency.
    createSellerReportURL(uuid, 'dbg_10USD_9USD'),
    createBidderReportURL(uuid, 'dbg_10USD_9USD')
  ]);
}, 'Converted currency use with sellerCurrency set matching.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionURLForHighestScoringOther(uuid),
    sellerCurrency: 'EUR'
  });
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '10???_0EUR'),
    createBidderReportURL(uuid, '10???_0EUR'),
    // sellerCurrency set, and no bid available in it: get 0s.
    createSellerReportURL(uuid, 'dbg_0EUR_0EUR'),
    createBidderReportURL(uuid, 'dbg_0EUR_0EUR')
  ]);
}, 'Converted currency use with sellerCurrency different, no conversion.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL:
        createDecisionURLForHighestScoringOther(uuid, 'converted = 3 * bid'),
    sellerCurrency: 'EUR'
  });
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, '10???_27EUR'),
    createBidderReportURL(uuid, '10???_27EUR'),
    // sellerCurrency set, converted bids.
    createSellerReportURL(uuid, 'dbg_30EUR_27EUR'),
    createBidderReportURL(uuid, 'dbg_30EUR_27EUR')
  ]);
}, 'Converted currency use with sellerCurrency different, conversion.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  let result =
      await runCurrencyComponentAuctionForHighestScoringOther(test, uuid, {
        componentConversion: `
          modified = bid + 1;
          modifiedCurrency = 'EUR';`,
        componentAuctionConfigOverrides: {sellerCurrency: 'EUR'}
      });
  expectSuccess(result);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_11EUR_0???'),
    createSellerReportURL(uuid, 'component_10???_0EUR'),
    createBidderReportURL(uuid, '10???_0EUR'),
    // forDebuggingOnly info w/sellerCurrency set relies on conversion;
    // but sellerCurrency is on component auction only.
    createBidderReportURL(uuid, 'dbg_0EUR_0EUR'),
    createSellerReportURL(uuid, 'dbg_component_0EUR_0EUR'),
    createSellerReportURL(uuid, 'dbg_top_11???_0???'),
  ]);
}, 'Modified bid does not act in place of incomingBidInSellerCurrency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  let result =
      await runCurrencyComponentAuctionForHighestScoringOther(test, uuid, {
        componentConversion: `
          modified = bid + 1;
          modifiedCurrency = 'EUR';
          converted = bid - 1;`,
        componentAuctionConfigOverrides: {sellerCurrency: 'EUR'}
      });
  expectSuccess(result);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_11EUR_0???'),
    createSellerReportURL(uuid, 'component_10???_8EUR'),
    createBidderReportURL(uuid, '10???_8EUR'),
    // Debug at component shows converted; top-level has no sellerCurrency,
    // so shows modified.
    createBidderReportURL(uuid, 'dbg_9EUR_8EUR'),
    createSellerReportURL(uuid, 'dbg_component_9EUR_8EUR'),
    createSellerReportURL(uuid, 'dbg_top_11???_0???'),
  ]);
}, 'Both modified bid and incomingBidInSellerCurrency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  let result =
      await runCurrencyComponentAuctionForHighestScoringOther(test, uuid, {
        componentConversion: `
          modified = bid + 1;
          modifiedCurrency = 'CAD';`,
        topLevelAuctionConfigOverrides: {sellerCurrency: 'EUR'},
        topLevelConversion: `converted = 3 * bid;`,
      });
  expectSuccess(result);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_11???_0???'),
    createSellerReportURL(uuid, 'component_10???_9???'),
    createBidderReportURL(uuid, '10???_9???'),
    // No sellerCurrency at component; debug at top-level shows the result of
    // conversion.
    createBidderReportURL(uuid, 'dbg_10???_9???'),
    createSellerReportURL(uuid, 'dbg_component_10???_9???'),
    createSellerReportURL(uuid, 'dbg_top_33EUR_0???'),
  ]);
}, 'incomingBidInSellerCurrency at top-level trying to convert is OK.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  let result =
      await runCurrencyComponentAuctionForHighestScoringOther(test, uuid, {
        componentConversion: `
          modified = bid + 1;
          modifiedCurrency = 'EUR';`,
        topLevelAuctionConfigOverrides: {sellerCurrency: 'EUR'},
        topLevelConversion: `converted = 3 * bid;`,
      });
  // Tried to change a bid that was already in EUR.
  expectNoWinner(result);
}, 'incomingBidInSellerCurrency at top-level trying to change bid is not OK.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinTwoGroupsForHighestScoringOther(test, uuid);
  let result =
      await runCurrencyComponentAuctionForHighestScoringOther(test, uuid, {
        componentConversion: `
          modified = bid + 1;
          modifiedCurrency = 'EUR';`,
        topLevelAuctionConfigOverrides: {sellerCurrency: 'EUR'},
        topLevelConversion: `converted = bid;`,
      });
  // Changing the bid to itself when it was already in right currency is OK.
  expectSuccess(result);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [
    createSellerReportURL(uuid, 'top_11???_0???'),
    createSellerReportURL(uuid, 'component_10???_9???'),
    createBidderReportURL(uuid, '10???_9???'),
    // No sellerCurrency at component; debug at top-level shows the result of
    // no-op conversion.
    createBidderReportURL(uuid, 'dbg_10???_9???'),
    createSellerReportURL(uuid, 'dbg_component_10???_9???'),
    createSellerReportURL(uuid, 'dbg_top_11EUR_0???'),
  ]);
}, 'incomingBidInSellerCurrency at top-level doing a no-op conversion OK.');

// TODO: PrivateAggregation. It follows the same rules as
// highestScoringOtherBid, but is actually visible at top-level.
