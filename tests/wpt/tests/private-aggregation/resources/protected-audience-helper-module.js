// This file is adapted from /fledge/tentative/resources/fledge-util.js,
// removing unnecessary logic and modifying to allow it to be run in the
// private-aggregation directory.

"use strict;"

const FULL_URL = window.location.href;
let BASE_URL = FULL_URL.substring(0, FULL_URL.lastIndexOf('/') + 1)
const BASE_PATH = (new URL(BASE_URL)).pathname;
const DEFAULT_INTEREST_GROUP_NAME = 'default name';

// Use python script files under fledge directory
const FLEDGE_DIR = '/fledge/tentative/';
const FLEDGE_BASE_URL = BASE_URL.replace(BASE_PATH, FLEDGE_DIR);

// Sleep method that waits for prescribed number of milliseconds.
const sleep = ms => new Promise(resolve => step_timeout(resolve, ms));

// Generates a UUID by token.
function generateUuid() {
  let uuid = token();
  return uuid;
}

// Creates a URL that will be sent to the handler.
// `uuid` is used to identify the stash shard to use.
// `operate` is used to set action as write or read.
// `report` is used to carry the message for write requests.
function createReportingURL(uuid, operation, report = 'default-report') {
  let url = new URL(`${window.location.origin}${BASE_PATH}resources/protected_audience_event_level_report_handler.py`);
  url.searchParams.append('uuid', uuid);
  url.searchParams.append('operation', operation);

  if (report)
    url.searchParams.append('report', report);

  return url.toString();
}

function createWritingURL(uuid, report) {
  return createReportingURL(uuid, 'write');
}

function createReadingURL(uuid) {
  return createReportingURL(uuid, 'read');
}

async function waitForObservedReports(uuid, expectedNumReports, timeout = 5000 /*ms*/) {
  expectedReports = Array(expectedNumReports).fill('default-report');
  const reportURL = createReadingURL(uuid);
  let startTime = performance.now();

  while (performance.now() - startTime < timeout) {
    let response = await fetch(reportURL, { credentials: 'omit', mode: 'cors' });
    let actualReports = await response.json();

    // If expected number of reports have been observed, compare with list of
    // all expected reports and exit.
    if (actualReports.length == expectedReports.length) {
      assert_array_equals(actualReports.sort(), expectedReports);
      return;
    }

    await sleep(/*ms=*/ 100);
  }
  assert_unreached("Report fetching timed out: " + uuid);
}

// Creates a bidding script with the provided code in the method bodies. The
// bidding script's generateBid() method will return a bid of 9 for the first
// ad, after the passed in code in the "generateBid" input argument has been
// run, unless it returns something or throws.
//
// The default reportWin() method is empty.
function createBiddingScriptURL(params = {}) {
  let url = new URL(`${FLEDGE_BASE_URL}resources/bidding-logic.sub.py`);
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
  let url = new URL(`${FLEDGE_BASE_URL}resources/decision-logic.sub.py`);
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
// be last.
function createRenderURL(uuid, script) {
  let url = new URL(`${FLEDGE_BASE_URL}resources/fenced-frame.sub.py`);
  if (script)
    url.searchParams.append('script', script);
  url.searchParams.append('uuid', uuid);
  return url.toString();
}

// Joins an interest group that, by default, is owned by the current frame's
// origin, is named DEFAULT_INTEREST_GROUP_NAME, has a bidding script that
// issues a bid of 9 with a renderURL of "https://not.checked.test/${uuid}".
// `interestGroupOverrides` is required to override fields in the joined
// interest group.
async function joinInterestGroup(test, uuid, interestGroupOverrides) {
  const INTEREST_GROUP_LIFETIME_SECS = 60;

  let interestGroup = {
    owner: window.location.origin,
    name: DEFAULT_INTEREST_GROUP_NAME,
    ads: [{renderURL: createRenderURL(uuid)}],
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
// body of scoreAd(), reportResult(), generateBid() and reportWin(), as well as
// additional arguments to be passed to joinAdInterestGroup() and runAdAuction()
async function runReportTest(test, uuid, codeToInsert,
                             expectedNumReports = 0, overrides = {}) {
  let generateBid = codeToInsert.generateBid;
  let scoreAd = codeToInsert.scoreAd;
  let reportWin = codeToInsert.reportWin;
  let reportResult = codeToInsert.reportResult;

  let extraInterestGroupOverrides = overrides.joinAdInterestGroup || {}
  let extraAuctionConfigOverrides = overrides.runAdAuction || {}

  let interestGroupOverrides = {
    biddingLogicURL: createBiddingScriptURL({ generateBid, reportWin }),
    ...extraInterestGroupOverrides
  };
  let auctionConfigOverrides = {
    decisionLogicURL: createDecisionScriptURL(
      uuid, { scoreAd, reportResult }),
    ...extraAuctionConfigOverrides
  }

  await joinInterestGroup(test, uuid, interestGroupOverrides);
  await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);

  if (expectedNumReports) {
    await waitForObservedReports(uuid, expectedNumReports);
  }
}
