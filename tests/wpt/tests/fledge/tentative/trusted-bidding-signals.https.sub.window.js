// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

"use strict";

// These tests focus on trustedBiddingSignals: Requesting them, handling network
// errors, handling the keys portion of the response, and passing keys to
// worklet scripts, and handling the Data-Version header
//
// Because of request batching, left over interest groups from
// other tests may result in tests that request TRUSTED_BIDDING_SIGNALS_URL
// with certain special keys failing, if interest groups with names other than
// the default one are not successfully left after previously run tests.

// Helper for trusted bidding signals test. Runs an auction, and fails the
// test if there's no winner. "generateBidCheck" is an expression that should
// be true when evaluated in generateBid(). "interestGroupOverrides" is a
// set of overridden fields added to the default interestGroup when joining it,
// allowing trusted bidding signals keys and URL to be set, in addition to other
// fields.
async function runTrustedBiddingSignalsTest(
    test, generateBidCheck, interestGroupOverrides = {}) {
  interestGroupOverrides.biddingLogicUrl =
      createBiddingScriptUrl({
          generateBid: `if (!(${generateBidCheck})) return false;` });
  await runBasicFledgeTestExpectingWinner(
      test, {interestGroupOverrides: interestGroupOverrides});
}

// Much like runTrustedBiddingSignalsTest, but runs auctions through reporting
// as well, and evaluates `check` both in generateBid() and reportWin(). Also
// makes sure browserSignals.dataVersion is undefined in scoreAd() and
// reportResult().
async function runTrustedBiddingSignalsDataVersionTest(
    test, check, interestGroupOverrides = {}) {
  const uuid = generateUuid(test);
  interestGroupOverrides.biddingLogicUrl =
      createBiddingScriptUrl({
          generateBid:
              `if (!(${check})) return false;`,
          reportWin:
              `if (!(${check}))
                sendReportTo('${createBidderReportUrl(uuid, 'error')}');
              else
                sendReportTo('${createBidderReportUrl(uuid)}');` });
  await joinInterestGroup(test, uuid, interestGroupOverrides);

  const auctionConfigOverrides = {
    decisionLogicUrl: createDecisionScriptUrl(
        uuid,
        { scoreAd:
              `if (browserSignals.dataVersion !== undefined)
                return false;`,
          reportResult:
              `if (browserSignals.dataVersion !== undefined)
                 sendReportTo('${createSellerReportUrl(uuid, 'error')}')
               sendReportTo('${createSellerReportUrl(uuid)}')`,
        })
  }
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(
      uuid, [createBidderReportUrl(uuid), createSellerReportUrl(uuid)]);
}

promise_test(async test => {
  await runTrustedBiddingSignalsTest(test, 'trustedBiddingSignals === null');
}, 'No trustedBiddingSignalsKeys or trustedBiddingSignalsUrl.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['numValue'] });
}, 'trustedBiddingSignalsKeys but no trustedBiddingSignalsUrl.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'trustedBiddingSignalsUrl without trustedBiddingSignalsKeys.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['close-connection'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'trustedBiddingSignalsUrl closes the connection without sending anything.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['http-error'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is HTTP 404 error.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['no-content-type'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no content-type.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['wrong-content-type'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has wrong content-type.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['fledge-not-allowed'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response does not allow fledge.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['bad-allow-fledge'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has wrong X-Allow-FLEDGE header.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['no-allow-fledge'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no X-Allow-FLEDGE header.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no body.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:Not JSON'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is not JSON.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:[]'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is a JSON array.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:{JSON_keys_need_quotes: 1}'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response in invalid JSON object.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["replace-body:{}"] === null',
      { trustedBiddingSignalsKeys: ['replace-body:{}'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no keys object.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, `trustedBiddingSignals['replace-body:{"keys":{}}'] === null`,
      { trustedBiddingSignalsKeys: ['replace-body:{"keys":{}}'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no keys.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["0"] === null &&
       trustedBiddingSignals["1"] === null &&
       trustedBiddingSignals["2"] === null &&
       trustedBiddingSignals["length"] === null`,
      { trustedBiddingSignalsKeys:
            ['replace-body:{"keys":[1,2,3]}', "0", "1", "2", "length"],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response keys is incorrectly an array.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["wrong-value"] === null &&
      trustedBiddingSignals["another-value"] === undefined`,
      { trustedBiddingSignalsKeys: ['wrong-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has key not in trustedBiddingSignalsKeys.');

promise_test(async test => {
    await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["null-value"] === null',
      { trustedBiddingSignalsKeys: ['null-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response null value for key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["num-value"] === 1',
      { trustedBiddingSignalsKeys: ['num-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has a number value for key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["string-value"] === "1"',
      { trustedBiddingSignalsKeys: ['string-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has string value for key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `JSON.stringify(trustedBiddingSignals["array-value"]) === '[1,"foo",null]'`,
      { trustedBiddingSignalsKeys: ['array-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has array value for key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `Object.keys(trustedBiddingSignals["object-value"]).length  === 2 &&
       trustedBiddingSignals["object-value"]["a"] === "b" &&
       JSON.stringify(trustedBiddingSignals["object-value"]["c"]) === '["d"]'`,
      { trustedBiddingSignalsKeys: ['object-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has object value for key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals[""] === "default value"',
      { trustedBiddingSignalsKeys: [''],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives empty string key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `Object.keys(trustedBiddingSignals).length === 6 &&
       trustedBiddingSignals["wrong-value"] === null &&
       trustedBiddingSignals["null-value"] === null &&
       trustedBiddingSignals["num-value"] === 1 &&
       trustedBiddingSignals["string-value"] === "1" &&
       JSON.stringify(trustedBiddingSignals["array-value"]) === '[1,"foo",null]' &&
       trustedBiddingSignals[""] === "default value"`,
      { trustedBiddingSignalsKeys: ['wrong-value', 'null-value', 'num-value',
                                    'string-value', 'array-value', ''],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has multiple keys.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals["+%20 \x00?,3#&"] === "default value"',
      { trustedBiddingSignalsKeys: ['+%20 \x00?,3#&'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives escaped key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals["\x00"] === "default value"',
      { trustedBiddingSignalsKeys: ['\x00'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives null key.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["interest-group-names"] === '["${DEFAULT_INTEREST_GROUP_NAME}"]'`,
      { trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives interest group name.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      // Interest group names is a JSONified list of JSONified names, so the
      // null ends up being escaped twice.
      `trustedBiddingSignals["interest-group-names"] === '["+%20 \\\\u0000?,3#&"]'`,
      { name: '+%20 \x00?,3#&',
        trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives escaped interest group name.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["interest-group-names"] === '[""]'`,
      { name: '',
        trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives empty interest group name.');

promise_test(async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["hostname"] === "${window.location.hostname}"`,
      { trustedBiddingSignalsKeys: ['hostname'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives hostname field.');

/////////////////////////////////////////////////////////////////////////////
// Data-Version tests
/////////////////////////////////////////////////////////////////////////////

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['num-value'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no data-version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 3',
      { trustedBiddingSignalsKeys: ['data-version:3'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has numeric Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 0',
      { trustedBiddingSignalsKeys: ['data-version:0'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has min Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 4294967295',
      { trustedBiddingSignalsKeys: ['data-version:4294967295'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has max Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:4294967296'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has too large Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:03'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has Data-Version with leading 0.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:-1'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has negative Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:1.3'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has decimal in Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:2 2'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has space in Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:0x4'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has hex Data-Version.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 4',
      { name: 'data-version',
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has Data-Version and no trustedBiddingSignalsKeys.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and empty body.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:[]'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and JSON array body.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:{} {}'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and double JSON object body.');

promise_test(async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 3',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:{"keys":5}'],
        trustedBiddingSignalsUrl: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and invalid keys entry');
