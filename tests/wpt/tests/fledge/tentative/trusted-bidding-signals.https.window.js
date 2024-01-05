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
// META: variant=?41-45
// META: variant=?46-50
// META: variant=?51-55
// META: variant=?56-60
// META: variant=?61-65
// META: variant=?66-last

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
    test, generateBidCheck, interestGroupOverrides = {}, auctionConfigOverrides = {}, uuidOverride = null) {
  interestGroupOverrides.biddingLogicURL =
      createBiddingScriptURL({
          allowComponentAuction: true,
          generateBid: `if (!(${generateBidCheck})) return false;` });
  let testConfig = {
    interestGroupOverrides: interestGroupOverrides,
    auctionConfigOverrides: auctionConfigOverrides
  };
  if (uuidOverride)
    testConfig.uuid = uuidOverride;
  await joinGroupAndRunBasicFledgeTestExpectingWinner(test, testConfig);
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

/////////////////////////////////////////////////////////////////////////////
// trustedBiddingSignalsSlotSizeMode tests
/////////////////////////////////////////////////////////////////////////////

async function runTrustedBiddingSignalsSlotSizeTest(
    test,
    expectedSlotSize,
    expectedAllSlotsRequestedSizes,
    trustedBiddingSignalsSlotSizeMode = null,
    auctionConfigOverrides = {},
    uuidOverride = null) {
  await runTrustedBiddingSignalsTest(
      test,
      `trustedBiddingSignals["slotSize"] ===
           ${JSON.stringify(expectedSlotSize)} &&
       trustedBiddingSignals["allSlotsRequestedSizes"] ===
           ${JSON.stringify(expectedAllSlotsRequestedSizes)}`,
      { trustedBiddingSignalsKeys: ['slotSize', 'allSlotsRequestedSizes'],
        trustedBiddingSignalsSlotSizeMode: trustedBiddingSignalsSlotSizeMode,
        trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL},
      auctionConfigOverrides,
      uuidOverride);
}

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found');
}, 'Null trustedBiddingSignalsSlotSizeMode, no sizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'not-a-real-mode');
}, 'Unknown trustedBiddingSignalsSlotSizeMode, no sizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'none');
}, 'none trustedBiddingSignalsSlotSizeMode, no sizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size');
}, 'slot-size trustedBiddingSignalsSlotSizeMode, no sizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size');
}, 'all-slots-requested-sizes trustedBiddingSignalsSlotSizeMode, no sizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'none',
      {requestedSize: {width:'10', height:'20'}});
}, 'none trustedBiddingSignalsSlotSizeMode, requestedSize in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/null,
      {requestedSize: {width:'10', height:'20'}});
}, 'Null trustedBiddingSignalsSlotSizeMode, requestedSize in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'not-a-real-mode',
      {requestedSize: {width:'10', height:'20'}});
}, 'Unknown trustedBiddingSignalsSlotSizeMode, requestedSize in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'10px,20px',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {requestedSize: {width:'10', height:'20'}});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, requestedSize in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'all-slots-requested-sizes',
      {requestedSize: {width:'10', height:'20'}});
}, 'all-slots-requested-sizes trustedBiddingSignalsSlotSizeMode, requestedSize in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'none',
      {allSlotsRequestedSizes: [{width:10, height:20}]});
}, 'none trustedBiddingSignalsSlotSizeMode, allSlotsRequestedSizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/null,
      {allSlotsRequestedSizes: [{width:'10', height:'20'}]});
}, 'Null trustedBiddingSignalsSlotSizeMode, allSlotsRequestedSizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'not-a-real-mode',
      {allSlotsRequestedSizes: [{width:'10', height:'20'}]});
}, 'Unknown trustedBiddingSignalsSlotSizeMode, allSlotsRequestedSizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {allSlotsRequestedSizes: [{width:'10', height:'20'}]});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, allSlotsRequestedSizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'10px,20px',
      /*trustedBiddingSignalsSlotSizeMode=*/'all-slots-requested-sizes',
      {allSlotsRequestedSizes: [{width:'10', height:'20'}]});
}, 'all-slots-requested-sizes trustedBiddingSignalsSlotSizeMode, allSlotsRequestedSizes in AuctionConfig');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'10px,20px',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {requestedSize: {width:'10px', height:'20px'}});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, explicit pixel units');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'80sw,12.5sh',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {requestedSize: {width:'80sw', height:'12.50sh'}});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, screen size units');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'80sh,12.5sw',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {requestedSize: {width:'80sh', height:'12.5sw'}});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, flipped screen size units');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'10px,25sh',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      {requestedSize: {width:'10', height:'25sh'}});
}, 'slot-size trustedBiddingSignalsSlotSizeMode, mixed pixel and screen width units');

subsetTest(promise_test, async test => {
  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'10px,20px,25sw,20px,22px,80sh',
      /*trustedBiddingSignalsSlotSizeMode=*/'all-slots-requested-sizes',
      { allSlotsRequestedSizes: [ {width:'10', height:'20'},
                                  {width:'25sw', height:'20px'},
                                  {width:'22', height:'80sh'}]});
}, 'all-slots-requested-sizes trustedBiddingSignalsSlotSizeMode, multiple unit types');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let group1ReportURL = createBidderReportURL(uuid, /*id=*/'none')
  let group2ReportURL = createBidderReportURL(uuid, /*id=*/'slot-size')
  let group3ReportURL = createBidderReportURL(uuid, /*id=*/'all-slots-requested-sizes')

  // The simplest way to make sure interest groups with different modes all receive
  // the right sizes is to have interest groups that modify their bids based on ad
  // size sent to the trusted server.
  await Promise.all(
      [ joinInterestGroup(
          test, uuid,
          { name: 'group 1',
            trustedBiddingSignalsKeys: ['slotSize', 'allSlotsRequestedSizes'],
            trustedBiddingSignalsSlotSizeMode: 'none',
            trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
            biddingLogicURL: createBiddingScriptURL(
                { generateBid:
                    `if (trustedBiddingSignals["slotSize"] !== "not-found" ||
                         trustedBiddingSignals["allSlotsRequestedSizes"] !== "not-found") {
                       throw "unexpected trustedBiddingSignals";
                     }
                     return {bid: 5, render: interestGroup.ads[0].renderURL};`,
                  reportWin: `sendReportTo("${group1ReportURL}");`})}),
        joinInterestGroup(
          test, uuid,
          { name: 'group 2',
            trustedBiddingSignalsKeys: ['slotSize', 'allSlotsRequestedSizes'],
            trustedBiddingSignalsSlotSizeMode: 'slot-size',
            trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
            biddingLogicURL: createBiddingScriptURL(
                { generateBid:
                    `if (trustedBiddingSignals["slotSize"] === "not-found" ||
                         trustedBiddingSignals["allSlotsRequestedSizes"] !== "not-found") {
                       throw "unexpected trustedBiddingSignals";
                     }
                     // Group 3 bids using the first digit of the first dimension.
                     return { bid: trustedBiddingSignals["slotSize"].substr(0, 1),
                              render: interestGroup.ads[0].renderURL};`,
                  reportWin: `sendReportTo("${group2ReportURL}");`})}),
        joinInterestGroup(
          test, uuid,
          { name: 'group 3',
            trustedBiddingSignalsKeys: ['slotSize', 'allSlotsRequestedSizes'],
            trustedBiddingSignalsSlotSizeMode: 'all-slots-requested-sizes',
            trustedBiddingSignalsURL: TRUSTED_BIDDING_SIGNALS_URL,
            biddingLogicURL: createBiddingScriptURL(
                { generateBid:
                    `if (trustedBiddingSignals["slotSize"] !== "not-found" ||
                         trustedBiddingSignals["allSlotsRequestedSizes"] === "not-found") {
                       throw "unexpected trustedBiddingSignals";
                     }
                     // Group 3 bids using the second digit of the first dimension.
                     return { bid: trustedBiddingSignals["allSlotsRequestedSizes"].substr(1, 1),
                              render: interestGroup.ads[0].renderURL};`,
                  reportWin: `sendReportTo("${group3ReportURL}");`})}),
      ]
  );

  let auctionConfigOverrides = {
    // Disable the default seller reporting, for simplicity.
    decisionLogicURL: createDecisionScriptURL(uuid, { reportResult: '' }),
    // Default sizes start with a "11", so groups 2 and 3 will start with a bid
    // of 1 and lose.
    requestedSize: {width:'11', height:'20'},
    allSlotsRequestedSizes: [{width:'11', height:'20'}]
  };

  // Group 1 wins the first auction.
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(uuid, [group1ReportURL]);

  // Group2 should bid "6" in the second auction, and win it.
  auctionConfigOverrides.requestedSize = {width:'61', height:'20'};
  auctionConfigOverrides.allSlotsRequestedSizes = [{width:'61', height:'20'}];
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(uuid, [group1ReportURL, group2ReportURL]);

  // Group3 should bid "7" in the third auction, and win it.
  auctionConfigOverrides.requestedSize = {width:'67', height:'20'};
  auctionConfigOverrides.allSlotsRequestedSizes = [{width:'67', height:'20'}];
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(uuid, [group1ReportURL, group2ReportURL, group3ReportURL]);
}, 'Mixed trustedBiddingSignalsSlotSizeModes in a single auction');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let componentAuctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [window.location.origin],
    requestedSize: {width:'10', height:'20'}
  };

  let auctionConfigOverrides = {
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig],
    requestedSize: {width:'22', height:'33'}
  }

  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      // Dimensions from the component auction should be used.
      /*expectedSlotSize=*/'10px,20px',
      /*expectedAllSlotsRequestedSizes=*/'not-found',
      /*trustedBiddingSignalsSlotSizeMode=*/'slot-size',
      auctionConfigOverrides,
      uuid);
}, 'slot-size trustedBiddingSignalsSlotSizeMode in a component auction');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let componentAuctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [window.location.origin],
    allSlotsRequestedSizes: [{width:'11', height:'22'}, {width:'12', height:'23'}]
  };

  let auctionConfigOverrides = {
    interestGroupBuyers: [],
    componentAuctions: [componentAuctionConfig],
    allSlotsRequestedSizes: [{width:'10', height:'20'}]
  }

  await runTrustedBiddingSignalsSlotSizeTest(
      test,
      // Dimensions from the component auction should be used.
      /*expectedSlotSize=*/'not-found',
      /*expectedAllSlotsRequestedSizes=*/'11px,22px,12px,23px',
      /*trustedBiddingSignalsSlotSizeMode=*/'all-slots-requested-sizes',
      auctionConfigOverrides,
      uuid);
}, 'all-slots-requested-sizes trustedBiddingSignalsSlotSizeMode in a component auction');
