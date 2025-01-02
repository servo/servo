// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last

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
      { reportResultSuccessCondition:
          reportResultSuccessCondition,
        reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');`,
        reportWinSuccessCondition:
          reportWinSuccessCondition,
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}

/////////////////////////////////////////////////////////////////////
// reportResult() to reportWin() message passing tests
/////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');
           return 45;`,
        reportWinSuccessCondition:
          'sellerSignals === 45',
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}, 'Seller passes number to bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');
           return 'foo';`,
        reportWinSuccessCondition:
          'sellerSignals === "foo"',
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}, 'Seller passes string to bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');
           return [3, 1, 2];`,
        reportWinSuccessCondition:
          'JSON.stringify(sellerSignals) === "[3,1,2]"',
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}, 'Seller passes array to bidder.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');
           return {a: 4, b:['c', null, {}]};`,
        reportWinSuccessCondition:
          `JSON.stringify(sellerSignals) === '{"a":4,"b":["c",null,{}]}'`,
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)]
  );
}, 'Seller passes object to bidder.');

/////////////////////////////////////////////////////////////////////
// reportResult() / reportWin() browserSignals tests.
/////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.topWindowHostname === "${window.location.hostname}"`,
    // reportWinSuccessCondition:
    `browserSignals.topWindowHostname === "${window.location.hostname}"`
  );
}, 'browserSignals.topWindowHostname test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.seller === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.seller === "${window.location.origin}"`
  );
}, 'browserSignals.seller test.');

subsetTest(promise_test, async test => {
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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.renderURL === "${createRenderURL(uuid)}"`,
    // reportWinSuccessCondition:
    `browserSignals.renderURL === "${createRenderURL(uuid)}"`,
    uuid
  );
}, 'browserSignals.renderURL test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.bid === 9`,
    // reportWinSuccessCondition:
    `browserSignals.bid === 9`
  );
}, 'browserSignals.bid test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.desirability === 18`,
    // reportWinSuccessCondition:
    `browserSignals.desirability === undefined`
  );
}, 'browserSignals.desirability test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.topLevelSellerSignals === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.topLevelSellerSignals === undefined`
  );
}, 'browserSignals.topLevelSellerSignals test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.dataVersion === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.dataVersion === undefined`
  );
}, 'browserSignals.dataVersion test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.modifiedBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.modifiedBid === undefined`
  );
}, 'browserSignals.modifiedBid test.');

subsetTest(promise_test, async test => {
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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: -2 }),
                            name: 'other interest group 1' });
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: -1 }),
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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: 2 }),
                            name: 'other interest group 1' });
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: 5 }),
                            name: 'other interest group 2' });
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: 2 }),
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

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.interestGroupName === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.interestGroupName === 'default name'`
  );
}, 'browserSignals.interestGroupName test.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === false`
  );
}, 'browserSignals.madeHighestScoringOtherBid with no other bids.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: -1 }),
                            name: 'other interest group 2' });
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.madeHighestScoringOtherBid === false`
  );
}, 'browserSignals.madeHighestScoringOtherBid with group that did not bid.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: 1 }),
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

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResultSuccessCondition:
          `browserSignals.reportingTimeout === undefined`,
        reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');`,
        reportWinSuccessCondition:
          'browserSignals.reportingTimeout === 100',
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      {reportingTimeout: 100});
}, 'browserSignals.reportingTimeout with custom value from auction config.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResultSuccessCondition:
          `browserSignals.reportingTimeout === undefined`,
        reportResult:
          `sendReportTo('${createSellerReportURL(uuid)}');`,
        reportWinSuccessCondition:
          'browserSignals.reportingTimeout === 5000',
        reportWin:
          `sendReportTo('${createBidderReportURL(uuid)}');` },
      // expectedReportURLs:
      [createSellerReportURL(uuid), createBidderReportURL(uuid)],
      // renderURLOverride
      null,
      // auctionConfigOverrides
      {reportingTimeout: 1234567890});
}, 'browserSignals.reportingTimeout above the cap value.');

subsetTest(promise_test, async test => {
  await runReportArgumentValidationTest(
    test,
    // reportResultSuccessCondition:
    `browserSignals.reportingTimeout === undefined`,
    // reportWinSuccessCondition:
    `browserSignals.reportingTimeout === 50`
  );
}, 'browserSignals.reportingTimeout default value.');
