// This file is adapted from /fledge/tentative/resources/fledge-util.js,
// removing unnecessary logic and modifying to allow it to be run in the
// private-aggregation directory.

"use strict;"

// Directory of fledge
const FLEDGE_DIR = '/fledge/tentative/';
const FULL_URL = window.location.href;
let BASE_URL = FULL_URL.substring(0, FULL_URL.lastIndexOf('/') + 1)
const BASE_PATH = (new URL(BASE_URL)).pathname;
const DEFAULT_INTEREST_GROUP_NAME = 'default name';

// Use python source files under fledge directory
BASE_URL = BASE_URL.replace(BASE_PATH, FLEDGE_DIR);

// Generates a UUID by token.
function generateUuid(test) {
  let uuid = token();
  return uuid;
}

// Creates a bidding script with the provided code in the method bodies. The
// bidding script's generateBid() method will return a bid of 9 for the first
// ad, after the passed in code in the "generateBid" input argument has been
// run, unless it returns something or throws.
//
// The default reportWin() method is empty.
function createBiddingScriptUrl(params = {}) {
  let url = new URL(`${BASE_URL}resources/bidding-logic.sub.py`);
  if (params.generateBid)
    url.searchParams.append('generateBid', params.generateBid);
  if (params.reportWin)
    url.searchParams.append('reportWin', params.reportWin);
  if (params.error)
    url.searchParams.append('error', params.error);
  if (params.bid)
    url.searchParams.append('bid', params.bid);
  return url.toString();
}

// Creates a decision script with the provided code in the method bodies. The
// decision script's scoreAd() method will reject ads with renderUrls that
// don't ends with "uuid", and will return a score equal to the bid, after the
// passed in code in the "scoreAd" input argument has been run, unless it
// returns something or throws.
//
// The default reportResult() method is empty.
function createDecisionScriptUrl(uuid, params = {}) {
  let url = new URL(`${BASE_URL}resources/decision-logic.sub.py`);
  url.searchParams.append('uuid', uuid);
  if (params.scoreAd)
    url.searchParams.append('scoreAd', params.scoreAd);
  if (params.reportResult)
    url.searchParams.append('reportResult', params.reportResult);
  if (params.error)
    url.searchParams.append('error', params.error);
  return url.toString();
}

// Creates a renderUrl for an ad that runs the passed in "script". "uuid" has
// no effect, beyond making the URL distinct between tests, and being verified
// by the decision logic script before accepting a bid. "uuid" is expected to
// be last.
function createRenderUrl(uuid, script) {
  let url = new URL(`${BASE_URL}resources/fenced-frame.sub.py`);
  if (script)
    url.searchParams.append('script', script);
  url.searchParams.append('uuid', uuid);
  return url.toString();
}

// Joins an interest group that, by default, is owned by the current frame's
// origin, is named DEFAULT_INTEREST_GROUP_NAME, has a bidding script that
// issues a bid of 9 with a renderUrl of "https://not.checked.test/${uuid}".
// `interestGroupOverrides` is required to override fields in the joined
// interest group.
async function joinInterestGroup(test, uuid, interestGroupOverrides) {
  const INTEREST_GROUP_LIFETIME_SECS = 60;

  let interestGroup = {
    owner: window.location.origin,
    name: DEFAULT_INTEREST_GROUP_NAME,
    ads: [{renderUrl: createRenderUrl(uuid)}],
    ...interestGroupOverrides
  };

  await navigator.joinAdInterestGroup(interestGroup,
                                      INTEREST_GROUP_LIFETIME_SECS);
  test.add_cleanup(
      async () => {await navigator.leaveAdInterestGroup(interestGroup)});
}

// Runs a FLEDGE auction and returns the result. `auctionConfigOverrides` is
// required to override fields in the auction configuration.
async function runBasicFledgeAuction(test, uuid, auctionConfigOverrides) {
  let auctionConfig = {
    seller: window.location.origin,
    interestGroupBuyers: [window.location.origin],
    resolveToConfig: true,
    ...auctionConfigOverrides
  };
  return await navigator.runAdAuction(auctionConfig);
}

// Calls runBasicFledgeAuction(), expecting the auction to have a winner.
// Creates a fenced frame that will be destroyed on completion of "test", and
// navigates it to the URN URL returned by the auction. Does not wait for the
// fenced frame to finish loading, since there's no API that can do that.
async function runBasicFledgeAuctionAndNavigate(test, uuid,
  auctionConfigOverrides) {
  let config = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from auction: ${config.constructor.type}`);

  let fencedFrame = document.createElement('fencedframe');
  fencedFrame.mode = 'opaque-ads';
  fencedFrame.config = config;
  document.body.appendChild(fencedFrame);
  test.add_cleanup(() => { document.body.removeChild(fencedFrame); });
}

// Joins an interest group and runs an auction, expecting no winner to be
// returned. "testConfig" can optionally modify the interest group or
// auctionConfig.
async function runBasicFledgeTestExpectingNoWinner(test, testConfig) {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, testConfig.interestGroupOverrides);
  let result = await runBasicFledgeAuction(
      test, uuid, testConfig.auctionConfigOverrides);
  assert_true(result === null, 'Auction unexpectedly had a winner');
}

// Test helper for report phase of auctions that lets the caller specify the
// body of reportResult() and reportWin().
//
// Passing worklets in null will cause the test fail.
//
// Null worklets test cases are handled under
// fledge.
async function runReportTest(test, uuid, reportResult, reportWin) {
  assert_not_equals(reportResult, null)
  assert_not_equals(reportWin, null)

  let interestGroupOverrides =
    { biddingLogicUrl: createBiddingScriptUrl({ reportWin }) };

  await joinInterestGroup(test, uuid, interestGroupOverrides);
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      { decisionLogicUrl: createDecisionScriptUrl(
        uuid, { reportResult })
    });
}
