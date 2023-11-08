// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=/common/subset-tests.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20
// META: variant=?21-25
// META: variant=?26-30
// META: variant=?31-35
// META: variant=?36-40
// META: variant=?41-last

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
  interestGroupOverrides.biddingLogicURL =
    createBiddingScriptURL({
          generateBid: `if (!(${generateBidCheck})) return false;` });
  await joinGroupAndRunBasicFledgeTestExpectingWinner(
      test, {interestGroupOverrides: interestGroupOverrides});
}

// Much like runTrustedBiddingSignalsTest, but runs auctions through reporting
// as well, and evaluates `check` both in generateBid() and reportWin(). Also
// makes sure browserSignals.dataVersion is undefined in scoreAd() and
// reportResult().
async function runTrustedBiddingSignalsDataVersionTest(
    test, check, interestGroupOverrides = {}) {
  const uuid = generateUuid(test);
  interestGroupOverrides.biddingLogicURL =
    createBiddingScriptURL({
          generateBid:
              `if (!(${check})) return false;`,
          reportWin:
              `if (!(${check}))
                sendReportTo('${createBidderReportURL(uuid, 'error')}');
              else
                sendReportTo('${createBidderReportURL(uuid)}');` });
  await joinInterestGroup(test, uuid, interestGroupOverrides);

  const auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { scoreAd:
              `if (browserSignals.dataVersion !== undefined)
                return false;`,
          reportResult:
              `if (browserSignals.dataVersion !== undefined)
                 sendReportTo('${createSellerReportURL(uuid, 'error')}')
               sendReportTo('${createSellerReportURL(uuid)}')`, })
  }
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(
      uuid, [createBidderReportURL(uuid), createSellerReportURL(uuid)]);
}

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(test, 'trustedBiddingSignals === null');
}, 'No trustedBiddingSignalsKeys or trustedBiddingSignalsURL.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['numValue'] });
}, 'trustedBiddingSignalsKeys but no trustedBiddingSignalsURL.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'trustedBiddingSignalsURL without trustedBiddingSignalsKeys.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['close-connection'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'trustedBiddingSignalsURL closes the connection without sending anything.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['http-error'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is HTTP 404 error.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['no-content-type'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no content-type.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['wrong-content-type'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has wrong content-type.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['ad-auction-not-allowed'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response does not allow fledge.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['bad-ad-auction-allowed'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has wrong Ad-Auction-Allowed header.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['no-ad-auction-allow'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no Ad-Auction-Allowed header.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no body.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:Not JSON'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is not JSON.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:[]'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response is a JSON array.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals === null',
      { trustedBiddingSignalsKeys: ['replace-body:{JSON_keys_need_quotes: 1}'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response in invalid JSON object.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["replace-body:{}"] === null',
      { trustedBiddingSignalsKeys: ['replace-body:{}'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no keys object.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, `trustedBiddingSignals['replace-body:{"keys":{}}'] === null`,
      { trustedBiddingSignalsKeys: ['replace-body:{"keys":{}}'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no keys.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["0"] === null &&
       trustedBiddingSignals["1"] === null &&
       trustedBiddingSignals["2"] === null &&
       trustedBiddingSignals["length"] === null`,
      { trustedBiddingSignalsKeys:
            ['replace-body:{"keys":[1,2,3]}', "0", "1", "2", "length"],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response keys is incorrectly an array.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["wrong-value"] === null &&
      trustedBiddingSignals["another-value"] === undefined`,
      { trustedBiddingSignalsKeys: ['wrong-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has key not in trustedBiddingSignalsKeys.');

subsetTest(promise_test, async test => {
    await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["null-value"] === null',
      { trustedBiddingSignalsKeys: ['null-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has null value for key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["num-value"] === 1',
      { trustedBiddingSignalsKeys: ['num-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has a number value for key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test, 'trustedBiddingSignals["string-value"] === "1"',
      { trustedBiddingSignalsKeys: ['string-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has string value for key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `JSON.stringify(trustedBiddingSignals["array-value"]) === '[1,"foo",null]'`,
      { trustedBiddingSignalsKeys: ['array-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has array value for key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `Object.keys(trustedBiddingSignals["object-value"]).length  === 2 &&
       trustedBiddingSignals["object-value"]["a"] === "b" &&
       JSON.stringify(trustedBiddingSignals["object-value"]["c"]) === '["d"]'`,
      { trustedBiddingSignalsKeys: ['object-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has object value for key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals[""] === "default value"',
      { trustedBiddingSignalsKeys: [''],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives empty string key.');

subsetTest(promise_test, async test => {
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
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has multiple keys.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals["+%20 \x00?,3#&"] === "default value"',
      { trustedBiddingSignalsKeys: ['+%20 \x00?,3#&'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives escaped key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      'trustedBiddingSignals["\x00"] === "default value"',
      { trustedBiddingSignalsKeys: ['\x00'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives null key.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["interest-group-names"] === '["${DEFAULT_INTEREST_GROUP_NAME}"]'`,
      { trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives interest group name.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      // Interest group names is a JSONified list of JSONified names, so the
      // null ends up being escaped twice.
      `trustedBiddingSignals["interest-group-names"] === '["+%20 \\\\u0000?,3#&"]'`,
      { name: '+%20 \x00?,3#&',
        trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives escaped interest group name.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["interest-group-names"] === '[""]'`,
      { name: '',
        trustedBiddingSignalsKeys: ['interest-group-names'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives empty interest group name.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["hostname"] === "${window.location.hostname}"`,
      { trustedBiddingSignalsKeys: ['hostname'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals receives hostname field.');

/////////////////////////////////////////////////////////////////////////////
// Data-Version tests
/////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['num-value'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has no Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 3',
      { trustedBiddingSignalsKeys: ['data-version:3'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has numeric Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 0',
      { trustedBiddingSignalsKeys: ['data-version:0'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has min Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 4294967295',
      { trustedBiddingSignalsKeys: ['data-version:4294967295'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has max Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:4294967296'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has too large Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:03'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has Data-Version with leading 0.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:-1'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has negative Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:1.3'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has decimal in Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:2 2'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has space in Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:0x4'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has hex Data-Version.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 4',
      { name: 'data-version',
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response has Data-Version and no trustedBiddingSignalsKeys.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and empty body.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:[]'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and JSON array body.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === undefined',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:{} {}'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and double JSON object body.');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsDataVersionTest(
      test,
      'browserSignals.dataVersion === 3',
      { trustedBiddingSignalsKeys: ['data-version:3', 'replace-body:{"keys":5}'],
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL });
}, 'Trusted bidding signals response with Data-Version and invalid keys entry');
