// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

// The tests in this file focus on simple auctions (one bidder, one seller, one
// origin, one frame) which have no winning bid, either due to errors or due to
// there being no bids, except where tests fit better with another set of tests.

// Errors common to bidding and decision logic scripts. These atrings will be
// appended to script URLs to make the python scripts that generate bidding
// logic and decision logic scripts with errors.
const COMMON_SCRIPT_ERRORS = [
  'error=close-connection',
  'error=http-error',
  'error=no-content-type',
  'error=wrong-content-type',
  'error=bad-allow-fledge',
  'error=fledge-not-allowed',
  'error=no-allow-fledge',
  'error=no-body',
];

const BIDDING_LOGIC_SCRIPT_ERRORS = [
  ...COMMON_SCRIPT_ERRORS,
  'error=no-generateBid',
  'generateBid=throw 1;',
  'generateBid=This does not compile',
  // Default timeout test. Doesn't check how long timing out takes.
  'generateBid=while(1);',
  // Bad return values:
  'generateBid=return 5;',
  'generateBid=return "Foo";',
  'generateBid=return interestGroup.ads[0].renderUrl;',
  'generateBid=return {bid: 1, render: "https://not-in-ads-array.test/"};',
  'generateBid=return {bid: 1};',
  'generateBid=return {render: interestGroup.ads[0].renderUrl};',
  // These are not bidding rather than errors.
  'generateBid=return {bid:0, render: interestGroup.ads[0].renderUrl};',
  'generateBid=return {bid:-1, render: interestGroup.ads[0].renderUrl};',
];

const DECISION_LOGIC_SCRIPT_ERRORS = [
  ...COMMON_SCRIPT_ERRORS,
  'error=no-scoreAd',
  'scoreAd=throw 1;',
  'scoreAd=This does not compile',
  // Default timeout test. Doesn't check how long timing out takes.
  'scoreAd=while(1);',
  // Bad return values:
  'scoreAd=return "Foo";',
  'scoreAd=return {desirability: "Foo"};',
  // These are rejecting the bid rather than errors.
  'scoreAd=return 0;',
  'scoreAd=return -1;',
  'scoreAd=return {desirability: 0};',
  'scoreAd=return {desirability: -1};',
];

for (error of BIDDING_LOGIC_SCRIPT_ERRORS) {
  promise_test((async (error, test) => {
    let biddingLogicUrl = `${BASE_URL}resources/bidding-logic.sub.py?${error}`;
    await runBasicFledgeTestExpectingNoWinner(
      test,
      {interestGroupOverrides: {biddingLogicUrl: biddingLogicUrl}}
    );
  }).bind(undefined, error), `Bidding logic script: ${error}`);
}

for (error of DECISION_LOGIC_SCRIPT_ERRORS) {
  promise_test((async (error, test) => {
    let decisionLogicUrl =
        `${BASE_URL}resources/decision-logic.sub.py?${error}`;
    await runBasicFledgeTestExpectingNoWinner(
      test, {auctionConfigOverrides: {decisionLogicUrl: decisionLogicUrl}}
    );
  }).bind(undefined, error), `Decision logic script: ${error}`);
}
