"use strict;"

const FULL_URL = window.location.href;
const BASE_URL = FULL_URL.substring(0, FULL_URL.lastIndexOf('/') + 1);
const BASE_PATH = (new URL(BASE_URL)).pathname;

const DEFAULT_INTEREST_GROUP_NAME = 'default name';

// Unlike other URLs, the trustedBiddingSignalsUrl can't have a query string
// that's set by tests, since FLEDGE controls it entirely, so tests that
// exercise it use a fixed URL string. Special keys and interest group names
// control the response.
const TRUSTED_BIDDING_SIGNALS_URL =
    `${BASE_URL}resources/trusted-bidding-signals.py`;

// Creates a URL that will be sent to the URL request tracker script.
// `uuid` is used to identify the stash shard to use.
// `dispatch` affects what the tracker script does.
// `id` can be used to uniquely identify tracked requests. It has no effect
//     on behavior of the script; it only serves to make the URL unique.
function createTrackerUrl(origin, uuid, dispatch, id = null) {
  let url = new URL(`${origin}${BASE_PATH}resources/request-tracker.py`);
  url.searchParams.append('uuid', uuid);
  url.searchParams.append('dispatch', dispatch);
  if (id)
    url.searchParams.append('id', id);
  return url.toString();
}

// Create tracked bidder/seller URLs. The only difference is the prefix added
// to the `id` passed to createTrackerUrl. The optional `id` field allows
// multiple bidder/seller report URLs to be distinguishable from each other.
function createBidderReportUrl(uuid, id = '1') {
  return createTrackerUrl(window.location.origin, uuid, `track_get`,
                          `bidder_report_${id}`);
}
function createSellerReportUrl(uuid, id = '1') {
  return createTrackerUrl(window.location.origin, uuid, `track_get`,
                          `seller_report_${id}`);
}

// Much like above ReportUrl methods, except designed for beacons, which
// are expected to be POSTs.
function createBidderBeaconUrl(uuid, id = '1') {
  return createTrackerUrl(window.location.origin, uuid, `track_post`,
                          `bidder_beacon_${id}`);
}
function createSellerBeaconUrl(uuid, id = '1') {
  return createTrackerUrl(window.location.origin, uuid, `track_post`,
                          `seller_beacon_${id}`);
}

// Generates a UUID and registers a cleanup method with the test fixture to
// request a URL from the request tracking script that clears all data
// associated with the generated uuid when requested.
function generateUuid(test) {
  let uuid = token();
  test.add_cleanup(async () => {
    let cleanupUrl = createTrackerUrl(window.location.origin, uuid, 'clean_up');
    let response = await fetch(cleanupUrl, {credentials: 'omit', mode: 'cors'});
    assert_equals(await response.text(), 'cleanup complete',
                  `Sever state cleanup failed`);
  });
  return uuid;
}

// Repeatedly requests "request_list" URL until exactly the entries in
// "expectedRequests" have been observed by the request tracker script (in
// any order, since report URLs are not guaranteed to be sent in any order).
//
// Elements of `expectedRequests` should either be URLs, in the case of GET
// requests, or "<URL>, body: <body>" in the case of POST requests.
//
// If any other strings are received from the tracking script, or the tracker
// script reports an error, fails the test.
async function waitForObservedRequests(uuid, expectedRequests) {
  let trackedRequestsUrl = createTrackerUrl(window.location.origin, uuid,
                                            'request_list');
  // Sort array for easier comparison, since order doesn't matter.
  expectedRequests.sort();
  while (true) {
    let response = await fetch(trackedRequestsUrl,
                               {credentials: 'omit', mode: 'cors'});
    let trackerData = await response.json();

    // Fail on fetch error.
    if (trackerData.error) {
      throw trackedRequestsUrl + ' fetch failed:' +
          JSON.stringify(trackerData);
    }

    // Fail on errors reported by the tracker script.
    if (trackerData.errors.length > 0) {
      throw 'Errors reported by request-tracker.py:' +
          JSON.stringify(trackerData.errors);
    }

    // If expected number of requests have been observed, compare with list of
    // all expected requests and exit.
    let trackedRequests = trackerData.trackedRequests;
    if (trackedRequests.length == expectedRequests.length) {
      assert_array_equals(trackedRequests.sort(), expectedRequests);
      break;
    }

    // If fewer than total number of expected requests have been observed,
    // compare what's been received so far, to have a greater chance to fail
    // rather than hang on error.
    for (const trackedRequest of trackedRequests) {
      assert_in_array(trackedRequest, expectedRequests);
    }
  }
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
// issues a bid of 9 with a renderUrl of "https://not.checked.test/${uuid}",
// and sends a report to createBidderReportUrl(uuid) if it wins. Waits for the
// join command to complete. Adds cleanup command to `test` to leave the
// interest group when the test completes.
//
// `interestGroupOverrides` may be used to override fields in the joined
// interest group.
async function joinInterestGroup(test, uuid, interestGroupOverrides = {},
                                 durationSeconds = 60) {
  let interestGroup = {
    owner: window.location.origin,
    name: DEFAULT_INTEREST_GROUP_NAME,
    biddingLogicUrl: createBiddingScriptUrl(
        { reportWin: `sendReportTo('${createBidderReportUrl(uuid)}');` }),
    ads: [{renderUrl: createRenderUrl(uuid)}],
    ...interestGroupOverrides
  };

  await navigator.joinAdInterestGroup(interestGroup, durationSeconds);
  test.add_cleanup(
      async () => {await navigator.leaveAdInterestGroup(interestGroup)});
}

// Similar to joinInterestGroup, but leaves the interest group instead.
// Generally does not need to be called manually when using
// "joinInterestGroup()".
async function leaveInterestGroup(interestGroupOverrides = {}) {
  let interestGroup = {
    owner: window.location.origin,
    name: DEFAULT_INTEREST_GROUP_NAME,
    ...interestGroupOverrides
  };

  await navigator.leaveAdInterestGroup(interestGroup);
}

// Runs a FLEDGE auction and returns the result. By default, the seller is the
// current frame's origin, and the only buyer is as well. The seller script
// rejects bids for URLs that don't contain "uuid" (to avoid running into issues
// with any interest groups from other tests), and reportResult() sends a report
// to createSellerReportUrl(uuid).
//
// `auctionConfigOverrides` may be used to override fields in the auction
// configuration.
async function runBasicFledgeAuction(test, uuid, auctionConfigOverrides = {}) {
  let auctionConfig = {
    seller: window.location.origin,
    decisionLogicUrl: createDecisionScriptUrl(
        uuid,
        { reportResult: `sendReportTo('${createSellerReportUrl(uuid)}');` }),
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
                                                auctionConfigOverrides = {}) {
  let config = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from auction: ${config.constructor.type}`);

  let fencedFrame = document.createElement('fencedframe');
  fencedFrame.mode = 'opaque-ads';
  fencedFrame.config = config;
  document.body.appendChild(fencedFrame);
  test.add_cleanup(() => { document.body.removeChild(fencedFrame); });
}

// Joins an interest group and runs an auction, expecting a winner to be
// returned. "testConfig" can optionally modify the interest group or
// auctionConfig.
async function runBasicFledgeTestExpectingWinner(test, testConfig = {}) {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, testConfig.interestGroupOverrides);
  let config = await runBasicFledgeAuction(
      test, uuid, testConfig.auctionConfigOverrides);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from auction: ${config.constructor.type}`);
}

// Joins an interest group and runs an auction, expecting no winner to be
// returned. "testConfig" can optionally modify the interest group or
// auctionConfig.
async function runBasicFledgeTestExpectingNoWinner(test, testConfig = {}) {
  const uuid = generateUuid(test);
  await joinInterestGroup(test, uuid, testConfig.interestGroupOverrides);
  let result = await runBasicFledgeAuction(
      test, uuid, testConfig.auctionConfigOverrides);
  assert_true(result === null, 'Auction unexpectedly had a winner');
}

// Test helper for report phase of auctions that lets the caller specify the
// body of reportResult() and reportWin(). Passing in null will cause there
// to be no reportResult() or reportWin() method.
//
// If the "SuccessCondition" fields are non-null and evaluate to false in
// the corresponding reporting method, the report is sent to an error URL.
// Otherwise, the corresponding 'reportResult' / 'reportWin' values are run.
//
// `renderUrlOverride` allows the ad URL of the joined InterestGroup to
// to be set by the caller.
//
// Requesting error report URLs causes waitForObservedRequests() to throw
// rather than hang.
async function runReportTest(test, uuid, reportResultSuccessCondition,
                             reportResult, reportWinSuccessCondition, reportWin,
                             expectedReportUrls, renderUrlOverride) {
  if (reportResultSuccessCondition) {
    reportResult = `if (!(${reportResultSuccessCondition})) {
                      sendReportTo('${createSellerReportUrl(uuid, 'error')}');
                      return false;
                    }
                    ${reportResult}`;
  }
  let decisionScriptUrlParams = {};
  if (reportResult !== null)
    decisionScriptUrlParams.reportResult = reportResult;
  else
    decisionScriptUrlParams.error = 'no-reportResult';

  if (reportWinSuccessCondition) {
    reportWin = `if (!(${reportWinSuccessCondition})) {
                   sendReportTo('${createSellerReportUrl(uuid, 'error')}');
                   return false;
                 }
                 ${reportWin}`;
  }
  let biddingScriptUrlParams = {};
  if (reportWin !== null)
    biddingScriptUrlParams.reportWin = reportWin;
  else
    biddingScriptUrlParams.error = 'no-reportWin';

  let interestGroupOverrides =
      { biddingLogicUrl: createBiddingScriptUrl(biddingScriptUrlParams) };
  if (renderUrlOverride)
    interestGroupOverrides.ads = [{renderUrl: renderUrlOverride}]

  await joinInterestGroup(test, uuid, interestGroupOverrides);
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      { decisionLogicUrl: createDecisionScriptUrl(
                              uuid, decisionScriptUrlParams) });
  await waitForObservedRequests(uuid, expectedReportUrls);
}
