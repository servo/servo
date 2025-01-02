// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long

"use strict;"

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': 1.99,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        // Possible stochastic rounding results for 1.99
        `browserSignals.adCost === 1.9921875 || browserSignals.adCost === 1.984375`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Check adCost is stochastically rounded with 8 bit mantissa and exponent.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'bid': 1.99,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        // Possible stochastic rounding results for 1.99
        `browserSignals.bid === 1.9921875 || browserSignals.bid === 1.984375`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Check bid is stochastically rounded with 8 bit mantissa and exponent.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { scoreAd:
        `return {desirability: 1.99,
                 allowComponentAuction: false}`,
      reportResultSuccessCondition:
        // Possible stochastic rounding results for 1.99
        `browserSignals.desirability === 1.9921875 || browserSignals.desirability === 1.984375`,
      reportResult:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Check desirability is stochastically rounded with 8 bit mantissa and exponent.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid,
    {
      biddingLogicURL: createBiddingScriptURL({ bid: 1.99 }),
      name: 'other interest group 1' });
  await runReportTest(
    test, uuid,
    { reportResultSuccessCondition:
        // Possible stochastic rounding results for 1.99
        `browserSignals.highestScoringOtherBid === 1.9921875 || browserSignals.highestScoringOtherBid === 1.984375`,
      reportResult:
      `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Check highestScoringOtherBid is stochastically rounded with 8 bit mantissa and exponent.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': 2,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        `browserSignals.adCost === 2`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Value is ignored as a non-valid floating-point number.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': 1E-46,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        `browserSignals.adCost === 0`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Value is rounded to 0 if value is greater than 0 and its exponent is less than -128.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': -1E-46,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        `browserSignals.adCost === -0`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Value is rounded to -0 if value is greater than 0 and its exponent is less than -128.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': 1E+39,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        `browserSignals.adCost === Infinity`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Value is rounded to Infinity if value is greater than 0 and its exponent is greater than 127.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `return {'adCost': -1E+39,
                 'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWinSuccessCondition:
        `browserSignals.adCost === -Infinity`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');` },
    // expectedReportURLs
    [createBidderReportURL(uuid)]
  );
}, 'Value is rounded to -Infinity if value is less than 0 and its exponent is greater than 127.');
