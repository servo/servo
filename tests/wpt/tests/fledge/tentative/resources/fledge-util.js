"use strict;"

const BASE_URL = document.baseURI.substring(0, document.baseURI.lastIndexOf('/') + 1);
const BASE_PATH = (new URL(BASE_URL)).pathname;
const RESOURCE_PATH = `${BASE_PATH}resources/`

const DEFAULT_INTEREST_GROUP_NAME = 'default name';

// Unlike other URLs, trusted signals URLs can't have query strings
// that are set by tests, since FLEDGE controls it entirely, so tests that
// exercise them use a fixed URL string. Note that FLEDGE adds query
// params when requesting these URLs, and the python scripts use these
// to construct the response.
const TRUSTED_BIDDING_SIGNALS_URL =
    `${BASE_URL}resources/trusted-bidding-signals.py`;
const TRUSTED_SCORING_SIGNALS_URL =
    `${BASE_URL}resources/trusted-scoring-signals.py`;

// Creates a URL that will be sent to the URL request tracker script.
// `uuid` is used to identify the stash shard to use.
// `dispatch` affects what the tracker script does.
// `id` can be used to uniquely identify tracked requests. It has no effect
//     on behavior of the script; it only serves to make the URL unique.
function createTrackerURL(origin, uuid, dispatch, id = null) {
  let url = new URL(`${origin}${BASE_PATH}resources/request-tracker.py`);
  url.searchParams.append('uuid', uuid);
  url.searchParams.append('dispatch', dispatch);
  if (id)
    url.searchParams.append('id', id);
  return url.toString();
}

// Create tracked bidder/seller URLs. The only difference is the prefix added
// to the `id` passed to createTrackerURL. The optional `id` field allows
// multiple bidder/seller report URLs to be distinguishable from each other.
function createBidderReportURL(uuid, id = '1', origin = window.location.origin) {
  return createTrackerURL(origin, uuid, `track_get`, `bidder_report_${id}`);
}
function createSellerReportURL(uuid, id = '1', origin = window.location.origin) {
  return createTrackerURL(origin, uuid, `track_get`, `seller_report_${id}`);
}

// Much like above ReportURL methods, except designed for beacons, which
// are expected to be POSTs.
function createBidderBeaconURL(uuid, id = '1', origin = window.location.origin) {
  return createTrackerURL(origin, uuid, `track_post`, `bidder_beacon_${id}`);
}
function createSellerBeaconURL(uuid, id = '1', origin = window.location.origin) {
  return createTrackerURL(origin, uuid, `track_post`, `seller_beacon_${id}`);
}

// Generates a UUID and registers a cleanup method with the test fixture to
// request a URL from the request tracking script that clears all data
// associated with the generated uuid when requested.
function generateUuid(test) {
  let uuid = token();
  test.add_cleanup(async () => {
    let cleanupURL = createTrackerURL(window.location.origin, uuid, 'clean_up');
    let response = await fetch(cleanupURL, {credentials: 'omit', mode: 'cors'});
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
  let trackedRequestsURL = createTrackerURL(window.location.origin, uuid,
                                            'request_list');
  // Sort array for easier comparison, since order doesn't matter.
  expectedRequests.sort();
  while (true) {
    let response = await fetch(trackedRequestsURL,
                               {credentials: 'omit', mode: 'cors'});
    let trackerData = await response.json();

    // Fail on fetch error.
    if (trackerData.error) {
      throw trackedRequestsURL + ' fetch failed:' +
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
      // Hide the uuid content in order to have a static expected file.
      assert_array_equals(trackedRequests.sort().map((url) =>
                            url.replace(uuid, '<uuid>')),
                            expectedRequests.map((url) =>
                            url.replace(uuid, '<uuid>')));
      break;
    }

    // If fewer than total number of expected requests have been observed,
    // compare what's been received so far, to have a greater chance to fail
    // rather than hang on error.
    for (const trackedRequest of trackedRequests) {
      assert_in_array(trackedRequest.replace(uuid, '<uuid>'),
                      expectedRequests.sort().map((url) =>
                      url.replace(uuid, '<uuid>')));
    }
  }
}

// Creates a bidding script with the provided code in the method bodies. The
// bidding script's generateBid() method will return a bid of 9 for the first
// ad, after the passed in code in the "generateBid" input argument has been
// run, unless it returns something or throws.
//
// The default reportWin() method is empty.
function createBiddingScriptURL(params = {}) {
  let origin = params.origin ? params.origin : new URL(BASE_URL).origin;
  let url = new URL(`${origin}${RESOURCE_PATH}bidding-logic.sub.py`);
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
// decision script's scoreAd() method will reject ads with renderURLs that
// don't ends with "uuid", and will return a score equal to the bid, after the
// passed in code in the "scoreAd" input argument has been run, unless it
// returns something or throws.
//
// The default reportResult() method is empty.
function createDecisionScriptURL(uuid, params = {}) {
  let origin = params.origin ? params.origin : new URL(BASE_URL).origin;
  let url = new URL(`${origin}${RESOURCE_PATH}decision-logic.sub.py`);
  url.searchParams.append('uuid', uuid);
  if (params.scoreAd)
    url.searchParams.append('scoreAd', params.scoreAd);
  if (params.reportResult)
    url.searchParams.append('reportResult', params.reportResult);
  if (params.error)
    url.searchParams.append('error', params.error);
  return url.toString();
}

// Creates a renderURL for an ad that runs the passed in "script". "uuid" has
// no effect, beyond making the URL distinct between tests, and being verified
// by the decision logic script before accepting a bid. "uuid" is expected to
// be last.  "signalsParams" also has no effect, but is used by
// trusted-scoring-signals.py to affect the response.
function createRenderURL(uuid, script, signalsParams) {
  let url = new URL(`${BASE_URL}resources/fenced-frame.sub.py`);
  if (script)
    url.searchParams.append('script', script);
  if (signalsParams)
    url.searchParams.append('signalsParams', signalsParams);
  url.searchParams.append('uuid', uuid);
  return url.toString();
}

// Creates an interest group owned by "origin" with a bidding logic URL located
// on "origin" as well. Uses standard render and report URLs, which are not
// necessarily on "origin". "interestGroupOverrides" may be used to override any
// field of the created interest group.
function createInterestGroupForOrigin(uuid, origin,
                                      interestGroupOverrides = {}) {
  return {
    owner: origin,
    name: DEFAULT_INTEREST_GROUP_NAME,
    biddingLogicURL: createBiddingScriptURL(
        { origin: origin,
          reportWin: `sendReportTo('${createBidderReportURL(uuid)}');` }),
    ads: [{ renderURL: createRenderURL(uuid) }],
    ...interestGroupOverrides
  };
}

// Joins an interest group that, by default, is owned by the current frame's
// origin, is named DEFAULT_INTEREST_GROUP_NAME, has a bidding script that
// issues a bid of 9 with a renderURL of "https://not.checked.test/${uuid}",
// and sends a report to createBidderReportURL(uuid) if it wins. Waits for the
// join command to complete. Adds cleanup command to `test` to leave the
// interest group when the test completes.
//
// `interestGroupOverrides` may be used to override fields in the joined
// interest group.
async function joinInterestGroup(test, uuid, interestGroupOverrides = {},
                                 durationSeconds = 60) {
  let interestGroup = createInterestGroupForOrigin(uuid, window.location.origin,
                                                   interestGroupOverrides);

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
// to createSellerReportURL(uuid).
//
// `auctionConfigOverrides` may be used to override fields in the auction
// configuration.
async function runBasicFledgeAuction(test, uuid, auctionConfigOverrides = {}) {
  let auctionConfig = {
    seller: window.location.origin,
    decisionLogicURL: createDecisionScriptURL(
        uuid,
        { reportResult: `sendReportTo('${createSellerReportURL(uuid)}');` }),
    interestGroupBuyers: [window.location.origin],
    resolveToConfig: true,
    ...auctionConfigOverrides
  };
  return await navigator.runAdAuction(auctionConfig);
}

// Wrapper around runBasicFledgeAuction() that runs an auction with the specified
// arguments, expecting the auction to have a winner. Returns the FencedFrameConfig
// from the auction.
async function runBasicFledgeTestExpectingWinner(test, uuid, auctionConfigOverrides = {}) {
  let config = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  assert_true(config !== null, `Auction unexpectedly had no winner`);
  assert_true(config instanceof FencedFrameConfig,
      `Wrong value type returned from auction: ${config.constructor.type}`);
  return config;
}

// Wrapper around runBasicFledgeAuction() that runs an auction with the specified
// arguments, expecting the auction to have no winner.
async function runBasicFledgeTestExpectingNoWinner(
    test, uuid, auctionConfigOverrides = {}) {
  let result = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  assert_true(result === null, 'Auction unexpectedly had a winner');
}

// Creates a fenced frame and applies fencedFrameConfig to it. Also adds a cleanup
// method to destroy the fenced frame at the end of the current test.
function createAndNavigateFencedFrame(test, fencedFrameConfig) {
  let fencedFrame = document.createElement('fencedframe');
  fencedFrame.mode = 'opaque-ads';
  fencedFrame.config = fencedFrameConfig;
  document.body.appendChild(fencedFrame);
  test.add_cleanup(() => { document.body.removeChild(fencedFrame); });
}

// Calls runBasicFledgeAuction(), expecting the auction to have a winner.
// Creates a fenced frame that will be destroyed on completion of "test", and
// navigates it to the URN URL returned by the auction. Does not wait for the
// fenced frame to finish loading, since there's no API that can do that.
async function runBasicFledgeAuctionAndNavigate(test, uuid,
                                                auctionConfigOverrides = {}) {
  let config = await runBasicFledgeTestExpectingWinner(test, uuid,
                                                       auctionConfigOverrides);
  createAndNavigateFencedFrame(test, config);
}

// Joins an interest group and runs an auction, expecting a winner to be
// returned. "testConfig" can optionally modify the uuid, interest group or
// auctionConfig.
async function joinGroupAndRunBasicFledgeTestExpectingWinner(test, testConfig = {}) {
  const uuid = testConfig.uuid ? testConfig.uuid : generateUuid(test);
  await joinInterestGroup(test, uuid, testConfig.interestGroupOverrides);
  await runBasicFledgeTestExpectingWinner(test, uuid, testConfig.auctionConfigOverrides);
}

// Joins an interest group and runs an auction, expecting no winner to be
// returned. "testConfig" can optionally modify the uuid, interest group or
// auctionConfig.
async function joinGroupAndRunBasicFledgeTestExpectingNoWinner(test, testConfig = {}) {
  const uuid = testConfig.uuid ? testConfig.uuid : generateUuid(test);
  await joinInterestGroup(test, uuid, testConfig.interestGroupOverrides);
  await runBasicFledgeTestExpectingNoWinner(test, uuid, testConfig.auctionConfigOverrides);
}

// Test helper for report phase of auctions that lets the caller specify the
// body of reportResult() and reportWin(). Passing in null will cause there
// to be no reportResult() or reportWin() method.
//
// If the "SuccessCondition" fields are non-null and evaluate to false in
// the corresponding reporting method, the report is sent to an error URL.
// Otherwise, the corresponding 'reportResult' / 'reportWin' values are run.
//
// `codeToInsert` is a JS object that contains the following fields to control
// the code generated for the auction worklet:
// scoreAd - function body for scoreAd() seller worklet function
// reportResult - function body for reportResult() seller worklet function
// generateBid - function body for generateBid() buyer worklet function
// reportWin - function body for reportWin() buyer worklet function
//
// Additionally the following fields can be added to check for errors during the
// execution of the corresponding worklets:
// reportWinSuccessCondition - boolean condition added to reportWin() in the
// buyer worklet that triggers a sendReportTo() to an 'error' URL if not met.
// reportResultSuccessCondition - boolean condition added to reportResult() in
// the seller worklet that triggers a sendReportTo() to an 'error' URL if not
// met.
//
// `renderURLOverride` allows the ad URL of the joined InterestGroup to
// to be set by the caller.
//
// Requesting error report URLs causes waitForObservedRequests() to throw
// rather than hang.
async function runReportTest(test, uuid, codeToInsert, expectedReportURLs,
                             renderURLOverride) {
  let scoreAd = codeToInsert.scoreAd;
  let reportResultSuccessCondition = codeToInsert.reportResultSuccessCondition;
  let reportResult = codeToInsert.reportResult;
  let generateBid = codeToInsert.generateBid;
  let reportWinSuccessCondition = codeToInsert.reportWinSuccessCondition;
  let reportWin = codeToInsert.reportWin;

  if (reportResultSuccessCondition) {
    reportResult = `if (!(${reportResultSuccessCondition})) {
                      sendReportTo('${createSellerReportURL(uuid, 'error')}');
                      return false;
                    }
                    ${reportResult}`;
  }
  let decisionScriptURLParams = {};

  if (scoreAd !== undefined) {
    decisionScriptURLParams.scoreAd = scoreAd;
  }

  if (reportResult !== null)
    decisionScriptURLParams.reportResult = reportResult;
  else
    decisionScriptURLParams.error = 'no-reportResult';

  if (reportWinSuccessCondition) {
    reportWin = `if (!(${reportWinSuccessCondition})) {
                   sendReportTo('${createSellerReportURL(uuid, 'error')}');
                   return false;
                 }
                 ${reportWin}`;
  }
  let biddingScriptURLParams = {};

  if (generateBid !== undefined) {
    biddingScriptURLParams.generateBid = generateBid;
  }

  if (reportWin !== null)
    biddingScriptURLParams.reportWin = reportWin;
  else
    biddingScriptURLParams.error = 'no-reportWin';

  let interestGroupOverrides =
      { biddingLogicURL: createBiddingScriptURL(biddingScriptURLParams) };
  if (renderURLOverride)
    interestGroupOverrides.ads = [{ renderURL: renderURLOverride }]

  await joinInterestGroup(test, uuid, interestGroupOverrides);
  await runBasicFledgeAuctionAndNavigate(
      test, uuid,
      { decisionLogicURL: createDecisionScriptURL(uuid, decisionScriptURLParams) });
  await waitForObservedRequests(uuid, expectedReportURLs);
}
