// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20
// META: variant=?21-25
// META: variant=?26-30
// META: variant=?31-35
// META: variant=?36-40
// META: variant=?41-45
// META: variant=?46-last

"use strict;"

// The tests in this file focus on simple auctions (one bidder, one seller, one
// origin, one frame) which have no winning bid, either due to errors or due to
// there being no bids, except where tests fit better with another set of tests.

// Errors common to Protected Audiences network requests. These strings will be
// appended to URLs to make the Python scripts that generate responses respond
// with errors.
const COMMON_NETWORK_ERRORS = [
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
  ...COMMON_NETWORK_ERRORS,
  'error=no-generateBid',
  'generateBid=throw 1;',
  'generateBid=This does not compile',
  // Default timeout test. Doesn't check how long timing out takes.
  'generateBid=while(1);',
  // Bad return values:
  'generateBid=return 5;',
  'generateBid=return "Foo";',
  'generateBid=return interestGroup.ads[0].renderURL;',
  'generateBid=return {bid: 1, render: "https://not-in-ads-array.test/"};',
  'generateBid=return {bid: 1};',
  'generateBid=return {render: interestGroup.ads[0].renderURL};',
  // These are not bidding rather than errors.
  'generateBid=return {bid:0, render: interestGroup.ads[0].renderURL};',
  'generateBid=return {bid:-1, render: interestGroup.ads[0].renderURL};'
];

const DECISION_LOGIC_SCRIPT_ERRORS = [
  ...COMMON_NETWORK_ERRORS,
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
  'scoreAd=return {desirability: -1};'
];

const BIDDING_WASM_HELPER_ERRORS = [
  ...COMMON_NETWORK_ERRORS,
  'error=not-wasm'
];

for (error of BIDDING_LOGIC_SCRIPT_ERRORS) {
  subsetTest(promise_test, (async (error, test) => {
    let biddingLogicURL = `${BASE_URL}resources/bidding-logic.sub.py?${error}`;
    await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test,
      {interestGroupOverrides: {biddingLogicURL: biddingLogicURL}}
    );
  }).bind(undefined, error), `Bidding logic script: ${error}`);
}

for (error of DECISION_LOGIC_SCRIPT_ERRORS) {
  subsetTest(promise_test, (async (error, test) => {
    let decisionLogicURL =
        `${BASE_URL}resources/decision-logic.sub.py?${error}`;
    await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test, { auctionConfigOverrides: { decisionLogicURL: decisionLogicURL } }
    );
  }).bind(undefined, error), `Decision logic script: ${error}`);
}

for (error of BIDDING_WASM_HELPER_ERRORS) {
  subsetTest(promise_test, (async (error, test) => {
    let biddingWasmHelperURL =
        `${BASE_URL}resources/wasm-helper.py?${error}`;
    await joinGroupAndRunBasicFledgeTestExpectingNoWinner(
      test, { interestGroupOverrides: { biddingWasmHelperURL: biddingWasmHelperURL } }
    );
  }).bind(undefined, error), `Bidding WASM helper: ${error}`);
}
