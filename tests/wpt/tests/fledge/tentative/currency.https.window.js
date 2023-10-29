// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-4
// META: variant=?5-last

'use strict;'

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
      {biddingLogicURL: createBiddingScriptURL({bidCurrency: 'USD'})});
  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'Returning bid with currency, configuration w/o currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid);
  await runBasicFledgeTestExpectingWinner(
      test, uuid, {perBuyerCurrencies: {'*': 'USD'}});
}, 'Returning bid w/o currency, configuration w/currency.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURL({bidCurrency: 'USD'})});
  await runBasicFledgeTestExpectingWinner(
      test, uuid, {perBuyerCurrencies: {'*': 'USD'}});
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
    interestGroupBuyers: [window.location.origin, OTHER_ORIGIN1],
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
    interestGroupBuyers: [window.location.origin, OTHER_ORIGIN1],
    perBuyerCurrencies: {}
  };
  auctionConfigOverrides.perBuyerCurrencies[window.location.origin] = 'USD';
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
    interestGroupBuyers: [window.location.origin, OTHER_ORIGIN1],
    perBuyerCurrencies: {}
  };
  auctionConfigOverrides.perBuyerCurrencies['*'] = 'EUR';
}, 'Different currencies for different origins, none match.');

// TODO: look at currency passed in to scoring.
// Conversion to uniform currency (integrate private aggregation to check)
// --- also the passthrough and can't modify rule for things already in it.
// Basic sellerCurrency checks (requires component auctions; can be
// pass-through or modified bid).
