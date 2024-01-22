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

// Other origins that should all be distinct from the main frame origin
// that the tests start with.
const OTHER_ORIGIN1 = 'https://{{hosts[alt][]}}:{{ports[https][0]}}';
const OTHER_ORIGIN2 = 'https://{{hosts[alt][]}}:{{ports[https][1]}}';
const OTHER_ORIGIN3 = 'https://{{hosts[][]}}:{{ports[https][1]}}';
const OTHER_ORIGIN4 = 'https://{{hosts[][www]}}:{{ports[https][0]}}';
const OTHER_ORIGIN5 = 'https://{{hosts[][www]}}:{{ports[https][1]}}';
const OTHER_ORIGIN6 = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}';
const OTHER_ORIGIN7 = 'https://{{hosts[alt][www]}}:{{ports[https][1]}}';

// Creates a URL that will be sent to the URL request tracker script.
// `uuid` is used to identify the stash shard to use.
// `dispatch` affects what the tracker script does.
// `id` can be used to uniquely identify tracked requests. It has no effect
//     on behavior of the script; it only serves to make the URL unique.
// `id` will always be the last query parameter.
function createTrackerURL(origin, uuid, dispatch, id = null) {
  let url = new URL(`${origin}${BASE_PATH}resources/request-tracker.py`);
  let search = `uuid=${uuid}&dispatch=${dispatch}`;
  if (id)
    search += `&id=${id}`;
  url.search = search;
  return url.toString();
}

// Create a URL that when fetches clears tracked URLs. Note that the origin
// doesn't matter - it will clean up all tracked URLs with the provided uuid,
// regardless of origin they were fetched from.
function createCleanupURL(uuid) {
  return createTrackerURL(window.location.origin, uuid, 'clean_up');
}

// Create tracked bidder/seller URLs. The only difference is the prefix added
// to the `id` passed to createTrackerURL. The optional `id` field allows
// multiple bidder/seller report URLs to be distinguishable from each other.
// `id` will always be the last query parameter.
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

function createDirectFromSellerSignalsURL(origin = window.location.origin) {
  let url = new URL(`${origin}${BASE_PATH}resources/direct-from-seller-signals.py`);
  return url.toString();
}

// Generates a UUID and registers a cleanup method with the test fixture to
// request a URL from the request tracking script that clears all data
// associated with the generated uuid when requested.
function generateUuid(test) {
  let uuid = token();
  test.add_cleanup(async () => {
    let response = await fetch(createCleanupURL(uuid),
                               {credentials: 'omit', mode: 'cors'});
    assert_equals(await response.text(), 'cleanup complete',
                  `Sever state cleanup failed`);
  });
  return uuid;
}

// Helper to fetch "tracked_data" URL to fetch all data recorded by the
// tracker URL associated with "uuid". Throws on error, including if
// the retrieved object's errors field is non-empty.
async function fetchTrackedData(uuid) {
  let trackedRequestsURL = createTrackerURL(window.location.origin, uuid,
                                            'tracked_data');
  let response = await fetch(trackedRequestsURL,
                             {credentials: 'omit', mode: 'cors'});
  let trackedData = await response.json();

  // Fail on fetch error.
  if (trackedData.error) {
    throw trackedRequestsURL + ' fetch failed:' + JSON.stringify(trackedData);
  }

  // Fail on errors reported by the tracker script.
  if (trackedData.errors.length > 0) {
    throw 'Errors reported by request-tracker.py:' +
        JSON.stringify(trackedData.errors);
  }

  return trackedData;
}

// Repeatedly requests "tracked_data" URL until exactly the entries in
// "expectedRequests" have been observed by the request tracker script (in
// any order, since report URLs are not guaranteed to be sent in any order).
//
// Elements of `expectedRequests` should either be URLs, in the case of GET
// requests, or "<URL>, body: <body>" in the case of POST requests.
//
// If any other strings are received from the tracking script, or the tracker
// script reports an error, fails the test.
async function waitForObservedRequests(uuid, expectedRequests) {
  // Sort array for easier comparison, as observed request order does not
  // matter, and replace UUID to print consistent errors on failure.
  expectedRequests = expectedRequests.sort().map((url) => url.replace(uuid, '<uuid>'));

  while (true) {
    let trackedData = await fetchTrackedData(uuid);

    // Clean up "trackedRequests" in same manner as "expectedRequests".
    let trackedRequests = trackedData.trackedRequests.sort().map(
                              (url) => url.replace(uuid, '<uuid>'));

    // If expected number of requests have been observed, compare with list of
    // all expected requests and exit.
    if (trackedRequests.length == expectedRequests.length) {
      assert_array_equals(trackedRequests, expectedRequests);
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
function createBiddingScriptURL(params = {}) {
  let origin = params.origin ? params.origin : new URL(BASE_URL).origin;
  let url = new URL(`${origin}${RESOURCE_PATH}bidding-logic.sub.py`);
  // These checks use "==" to ignore null and not provided arguments, while
  // treating '' as a valid argument.
  if (params.generateBid != null)
    url.searchParams.append('generateBid', params.generateBid);
  if (params.reportWin != null)
    url.searchParams.append('reportWin', params.reportWin);
  if (params.error != null)
    url.searchParams.append('error', params.error);
  if (params.bid != null)
    url.searchParams.append('bid', params.bid);
  if (params.bidCurrency != null)
    url.searchParams.append('bidCurrency', params.bidCurrency);
  if (params.allowComponentAuction != null)
    url.searchParams.append('allowComponentAuction', JSON.stringify(params.allowComponentAuction))
  return url.toString();
}

// TODO: Make this return a valid WASM URL.
function createBiddingWasmHelperURL(params = {}) {
  let origin = params.origin ? params.origin : new URL(BASE_URL).origin;
  return `${origin}${RESOURCE_PATH}bidding-wasmlogic.wasm`;
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
  // These checks use "==" to ignore null and not provided arguments, while
  // treating '' as a valid argument.
  if (params.scoreAd != null)
    url.searchParams.append('scoreAd', params.scoreAd);
  if (params.reportResult != null)
    url.searchParams.append('reportResult', params.reportResult);
  if (params.error != null)
    url.searchParams.append('error', params.error);
  return url.toString();
}

// Creates a renderURL for an ad that runs the passed in "script". "uuid" has
// no effect, beyond making the URL distinct between tests, and being verified
// by the decision logic script before accepting a bid. "uuid" is expected to
// be last.  "signalsParams" also has no effect, but is used by
// trusted-scoring-signals.py to affect the response.
function createRenderURL(uuid, script, signalsParams, origin) {
  // These checks use "==" to ignore null and not provided arguments, while
  // treating '' as a valid argument.
  if (origin == null)
    origin = new URL(BASE_URL).origin;
  let url = new URL(`${origin}${RESOURCE_PATH}fenced-frame.sub.py`);
  if (script != null)
    url.searchParams.append('script', script);
  if (signalsParams != null)
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

// Checks that await'ed return value of runAdAuction() denotes a successful
// auction with a winner.
function expectSuccess(config) {
  assert_true(config !== null, `Auction unexpectedly had no winner`);
  assert_true(
      config instanceof FencedFrameConfig,
      `Wrong value type returned from auction: ${config.constructor.type}`);
}

// Checks that await'ed return value of runAdAuction() denotes an auction
// without a winner (but no fatal error).
function expectNoWinner(result) {
  assert_true(result === null, 'Auction unexpectedly had a winner');
}

// Wrapper around runBasicFledgeAuction() that runs an auction with the specified
// arguments, expecting the auction to have a winner. Returns the FencedFrameConfig
// from the auction.
async function runBasicFledgeTestExpectingWinner(test, uuid, auctionConfigOverrides = {}) {
  let config = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  expectSuccess(config);
  return config;
}

// Wrapper around runBasicFledgeAuction() that runs an auction with the specified
// arguments, expecting the auction to have no winner.
async function runBasicFledgeTestExpectingNoWinner(
    test, uuid, auctionConfigOverrides = {}) {
  let result = await runBasicFledgeAuction(test, uuid, auctionConfigOverrides);
  expectNoWinner(result);
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
// reportResultSuccessCondition - Success condition to trigger reportResult()
// reportResult - function body for reportResult() seller worklet function
// generateBid - function body for generateBid() buyer worklet function
// reportWinSuccessCondition - Success condition to trigger reportWin()
// decisionScriptURLOrigin - Origin of decision script URL
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
// `auctionConfigOverrides` may be used to override fields in the auction
// configuration.
//
// Requesting error report URLs causes waitForObservedRequests() to throw
// rather than hang.
async function runReportTest(test, uuid, codeToInsert, expectedReportURLs,
    renderURLOverride, auctionConfigOverrides) {
  let scoreAd = codeToInsert.scoreAd;
  let reportResultSuccessCondition = codeToInsert.reportResultSuccessCondition;
  let reportResult = codeToInsert.reportResult;
  let generateBid = codeToInsert.generateBid;
  let reportWinSuccessCondition = codeToInsert.reportWinSuccessCondition;
  let reportWin = codeToInsert.reportWin;
  let decisionScriptURLOrigin = codeToInsert.decisionScriptURLOrigin;

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

  if (decisionScriptURLOrigin !== undefined) {
    decisionScriptURLParams.origin = decisionScriptURLOrigin;
  }

  if (reportWinSuccessCondition) {
    reportWin = `if (!(${reportWinSuccessCondition})) {
                   sendReportTo('${createBidderReportURL(uuid, 'error')}');
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

  if (auctionConfigOverrides === undefined) {
    auctionConfigOverrides =
        { decisionLogicURL: createDecisionScriptURL(uuid, decisionScriptURLParams) };
  } else if (auctionConfigOverrides.decisionLogicURL === undefined) {
    auctionConfigOverrides.decisionLogicURL =
        createDecisionScriptURL(uuid, decisionScriptURLParams);
  }

  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
  await waitForObservedRequests(uuid, expectedReportURLs);
}

// Runs "script" in "child_window" via an eval call. The "child_window" must
// have been created by calling "createFrame()" below. "param" is passed to the
// context "script" is run in, so can be used to pass objects that
// "script" references that can't be serialized to a string, like
// fencedFrameConfigs.
async function runInFrame(test, child_window, script, param) {
  const messageUuid = generateUuid(test);
  let receivedResponse = {};

  let promise = new Promise(function(resolve, reject) {
    function WaitForMessage(event) {
      if (event.data.messageUuid != messageUuid)
        return;
      receivedResponse = event.data;
      if (event.data.result === 'success') {
        resolve();
      } else {
        reject(event.data.result);
      }
    }
    window.addEventListener('message', WaitForMessage);
    child_window.postMessage(
        {messageUuid: messageUuid, script: script, param: param}, '*');
  });
  await promise;
  return receivedResponse.returnValue;
}

// Creates an frame and navigates it to a URL on "origin", and waits for the URL
// to finish loading by waiting for the frame to send an event. Then returns
// the frame's Window object. Depending on the value of "is_iframe", the created
// frame will either be a new iframe, or a new top-level main frame. In the iframe
// case, its "allow" field will be set to "permissions".
//
// Also adds a cleanup callback to "test", which runs all cleanup functions
// added within the frame and waits for them to complete, and then destroys the
// iframe or closes the window.
async function createFrame(test, origin, is_iframe = true, permissions = null) {
  const frameUuid = generateUuid(test);
  const frameUrl =
      `${origin}${RESOURCE_PATH}subordinate-frame.sub.html?uuid=${frameUuid}`;
  let promise = new Promise(function(resolve, reject) {
    function WaitForMessage(event) {
      if (event.data.messageUuid != frameUuid)
        return;
      if (event.data.result === 'load complete') {
        resolve();
      } else {
        reject(event.data.result);
      }
    }
    window.addEventListener('message', WaitForMessage);
  });

  if (is_iframe) {
    let iframe = document.createElement('iframe');
    if (permissions)
      iframe.allow = permissions;
    iframe.src = frameUrl;
    document.body.appendChild(iframe);

    test.add_cleanup(async () => {
      await runInFrame(test, iframe.contentWindow, "await test_instance.do_cleanup();");
      document.body.removeChild(iframe);
    });

    await promise;
    return iframe.contentWindow;
  }

  let child_window = window.open(frameUrl);
  test.add_cleanup(async () => {
    await runInFrame(test, child_window, "await test_instance.do_cleanup();");
    child_window.close();
  });

  await promise;
  return child_window;
}

// Wrapper around createFrame() that creates an iframe and optionally sets
// permissions.
async function createIframe(test, origin, permissions = null) {
  return await createFrame(test, origin, /*is_iframe=*/true, permissions);
}

// Wrapper around createFrame() that creates a top-level window.
async function createTopLevelWindow(test, origin) {
  return await createFrame(test, origin, /*is_iframe=*/false);
}

// Joins a cross-origin interest group. Currently does this by joining the
// interest group in an iframe, though it may switch to using a .well-known
// fetch to allow the cross-origin join, when support for that is added
// to these tests, so callers should not assume that's the mechanism in use.
async function joinCrossOriginInterestGroup(test, uuid, origin, interestGroupOverrides = {}) {
  let interestGroup = JSON.stringify(
      createInterestGroupForOrigin(uuid, origin, interestGroupOverrides));

  let iframe = await createIframe(test, origin, 'join-ad-interest-group');
  await runInFrame(test, iframe,
                   `await joinInterestGroup(test_instance, "${uuid}", ${interestGroup})`);
}

// Joins an interest group in a top-level window, which has the same origin
// as the joined interest group.
async function joinInterestGroupInTopLevelWindow(
    test, uuid, origin, interestGroupOverrides = {}) {
  let interestGroup = JSON.stringify(
      createInterestGroupForOrigin(uuid, origin, interestGroupOverrides));

  let topLeveWindow = await createTopLevelWindow(test, origin);
  await runInFrame(test, topLeveWindow,
                   `await joinInterestGroup(test_instance, "${uuid}", ${interestGroup})`);
}

// Fetch directFromSellerSignals from seller and check header
// 'Ad-Auction-Signals' is hidden from documents.
async function fetchDirectFromSellerSignals(headers_content, origin) {
  const response = await fetch(
      createDirectFromSellerSignalsURL(origin),
      { adAuctionHeaders: true, headers: headers_content });

  if (!('Negative-Test-Option' in headers_content)) {
    assert_equals(
        response.status,
        200,
        'Failed to fetch directFromSellerSignals: ' + await response.text());
  }
  assert_false(
      response.headers.has('Ad-Auction-Signals'),
      'Header "Ad-Auction-Signals" should be hidden from documents.');
}

// Generate directFromSellerSignals evaluation code for different worklets and
// pass to `runReportTest()` as `codeToInsert`.
function directFromSellerSignalsValidatorCode(uuid, expectedSellerSignals,
    expectedAuctionSignals, expectedPerBuyerSignals) {
  expectedSellerSignals = JSON.stringify(expectedSellerSignals);
  expectedAuctionSignals = JSON.stringify(expectedAuctionSignals);
  expectedPerBuyerSignals = JSON.stringify(expectedPerBuyerSignals);

  return {
    // Seller worklets
    scoreAd:
      `if (directFromSellerSignals === null ||
           directFromSellerSignals.sellerSignals !== ${expectedSellerSignals} ||
           directFromSellerSignals.auctionSignals !== ${expectedAuctionSignals} ||
           Object.keys(directFromSellerSignals).length != 2) {
              throw 'Failed to get expected directFromSellerSignals in scoreAd(): ' +
                JSON.stringify(directFromSellerSignals);
          }`,
    reportResultSuccessCondition:
      `directFromSellerSignals !== null &&
           directFromSellerSignals.sellerSignals === ${expectedSellerSignals} &&
           directFromSellerSignals.auctionSignals === ${expectedAuctionSignals} &&
           Object.keys(directFromSellerSignals).length == 2`,
    reportResult:
      `sendReportTo("${createSellerReportURL(uuid)}");`,

    // Bidder worklets
    generateBid:
      `if (directFromSellerSignals === null ||
           directFromSellerSignals.perBuyerSignals !== ${expectedPerBuyerSignals} ||
           directFromSellerSignals.auctionSignals !== ${expectedAuctionSignals} ||
           Object.keys(directFromSellerSignals).length != 2) {
              throw 'Failed to get expected directFromSellerSignals in generateBid(): ' +
                JSON.stringify(directFromSellerSignals);
        }`,
    reportWinSuccessCondition:
      `directFromSellerSignals !== null &&
           directFromSellerSignals.perBuyerSignals === ${expectedPerBuyerSignals} &&
           directFromSellerSignals.auctionSignals === ${expectedAuctionSignals} &&
           Object.keys(directFromSellerSignals).length == 2`,
    reportWin:
      `sendReportTo("${createBidderReportURL(uuid)}");`,
  };
}
