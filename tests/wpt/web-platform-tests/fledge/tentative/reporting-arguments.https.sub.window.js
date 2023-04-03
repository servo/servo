// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

"use strict;"

// Simplified version of reportTest() for validating arguments to reporting
// methods. Only takes expressions to check in reporting methods. "uuid" is
// optional, and one is generated if not passed one.
async function runReportArgumentValidationTest(
    test, reportResultSuccessCondition, reportWinSuccessCondition, uuid) {
  if (!uuid)
    uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      reportResultSuccessCondition,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      reportWinSuccessCondition,
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}

/////////////////////////////////////////////////////////////////////
// reportResult() to reportWin() message passing tests
/////////////////////////////////////////////////////////////////////

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');
      return 45;`,
      // reportWin:
      'sellerSignals === 45',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Seller passes number to bidder.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');
      return 'foo';`,
      // reportWin:
      'sellerSignals === "foo"',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Seller passes string to bidder.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');
      return [3, 1, 2];`,
      // reportWin:
      'JSON.stringify(sellerSignals) === "[3,1,2]"',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Seller passes array to bidder.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');
      return {a: 4, b:['c', null, {}]};`,
      // reportWin:
      `JSON.stringify(sellerSignals) === '{"a":4,"b":["c",null,{}]}'`,
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Seller passes object to bidder.');

/////////////////////////////////////////////////////////////////////
// reportResult() / reportWin() browserSignals tests.
/////////////////////////////////////////////////////////////////////

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.topWindowHostname === "${window.location.hostname}"`,
    // reportWinSuccessCondition:
    `browserSignals.topWindowHostname === "${window.location.hostname}"`
  );
}, 'browserSignals.topWindowHostname test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.seller === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.seller === "${window.location.origin}"`
  );
}, 'browserSignals.seller test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.topLevelSeller === undefined &&
     browserSignals.componentSeller === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.topLevelSeller === undefined &&
     browserSignals.componentSeller === undefined`
  );
}, 'browserSignals.topLevelSeller and browserSignals.componentSeller test.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.renderUrl === "${createRenderUrl(uuid)}"`,
    // reportWinSuccessCondition:
    `browserSignals.renderUrl === "${createRenderUrl(uuid)}"`,
    uuid
  );
}, 'browserSignals.renderUrl test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.bid === 9`,
    // reportWinSuccessCondition:
    `browserSignals.bid === 9`
  );
}, 'browserSignals.bid test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.desirability === 18`,
    // reportWinSuccessCondition:
    `browserSignals.desirability === undefined`
  );
}, 'browserSignals.desirability test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.topLevelSellerSignals === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.topLevelSellerSignals === undefined`
  );
}, 'browserSignals.topLevelSellerSignals test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.dataVersion === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.dataVersion === undefined`
  );
}, 'browserSignals.dataVersion test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.modifiedBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.modifiedBid === undefined`
  );
}, 'browserSignals.modifiedBid test.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.highestScoringOtherBid === 0`,
    // reportWinSuccessCondition:
    `browserSignals.highestScoringOtherBid === 0`,
    uuid
  );
}, 'browserSignals.highestScoringOtherBid with no other interest groups test.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: -2}),
                            name: 'other interest group 1' });
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: -1}),
                            name: 'other interest group 2' });
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.highestScoringOtherBid === 0`,
    // reportWinSuccessCondition:
    `browserSignals.highestScoringOtherBid === 0`,
    uuid
  );
}, 'browserSignals.highestScoringOtherBid with other groups that do not bid.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: 2}),
                            name: 'other interest group 1' });
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: 5}),
                            name: 'other interest group 2' });
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: 2}),
                            name: 'other interest group 3' });
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.highestScoringOtherBid === 5`,
    // reportWinSuccessCondition:
    `browserSignals.highestScoringOtherBid === 5`,
    uuid
  );
}, 'browserSignals.highestScoringOtherBid with other bids.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.interestGroupName === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.interestGroupName === "default name"`
  );
}, 'browserSignals.interestGroupName test.');

promise_test(async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === false`
  );
}, 'browserSignals.madeHighestScoringOtherBid with no other bids.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
                          { biddingLogicUrl: createBiddingScriptUrl({bid: -1}),
                            name: 'other interest group 2' });
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === false`
  );
}, 'browserSignals.madeHighestScoringOtherBid with group that did not bid.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    { biddingLogicUrl: createBiddingScriptUrl({bid: 1}),
      name: 'other interest group 2' });
await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === true`,
    uuid
  );
}, 'browserSignals.madeHighestScoringOtherBid with other bid.');
