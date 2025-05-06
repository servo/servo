// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/ba-fledge-util.sub.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: script=third_party/cbor-js/cbor.js
// META: timeout=long
// META: variant=?1-last

"use strict";

// These tests focus on the debugReports field in AuctionConfig's
// serverResponse, i.e. auctions involving forDebuggingOnly reports.
// NOTE: Due to DB's fDO lockout/cooldown tables are not cleaned after each run
// of a test, fDO reports will not be sent after the first run of the test sent
// it, so we cannot reliably test fDO reports being sent when downsampling is
// enabled yet.

function createDebugReport(
    url, isWinReport = null, isSellerReport = null, componentWin = null) {
  let report = {};
  if (url !== null) {
    report.url = url;
  }
  if (isWinReport !== null) {
    report.isWinReport = isWinReport;
  }
  if (isSellerReport !== null) {
    report.isSellerReport = isSellerReport;
  }
  if (componentWin !== null) {
    report.componentWin = componentWin;
  }
  return report;
}

function createDebugReportsPerOrigin(
    adTechOrigin = MAIN_ORIGIN, reports = null) {
  let reportsPerOrigin = {};
  if (adTechOrigin !== null) {
    reportsPerOrigin.adTechOrigin = adTechOrigin;
  }
  if (reports !== null) {
    reportsPerOrigin.reports = reports;
  }
  return reportsPerOrigin;
}

const delay = ms => new Promise(resolve => step_timeout(resolve, ms));

// No forDebuggingOnly requests are observed, until time out.
async function noRequestsObserved(uuid, timeout = 2000 /*ms*/) {
  const endTime = performance.now() + timeout;

  do {
    let trackedData = await fetchTrackedData(uuid);
    // Replace UUID to print consistent errors on failure.
    let trackedRequests =
        trackedData.trackedRequests.map((url) => url.replace(uuid, '<uuid>'))
            .sort();

    // No forDebuggingOnly requests should be observed.
    for (const request of trackedRequests) {
      assert_false(
          request.includes('forDebuggingOnly'),
          'Unexpected forDebuggingOnly request: ' + request);
    }
    await delay(/*ms=*/ 100);
  } while (performance.now() < endTime);
}

async function testInvalidDebugReportsFields(
    test, uuid, debugReports, ownerOverride = null) {
  let result = await BA.testWithMutatedServerResponse(
      test, /*expectWin=*/ true,
      (msg) => {
        msg.debugReports = debugReports;
      },
      (ig, uuid) => {
        ig.ads[0].renderURL = createRenderURL(uuid);
      },
      ownerOverride);
  createAndNavigateFencedFrame(test, result);
  await noRequestsObserved(uuid);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly');
  let debugReports = [createDebugReportsPerOrigin(
      /*adTechOrigin=*/ null,
      [createDebugReport(bidderDebugReportURL, /*isWinReport=*/ true)])];
  await testInvalidDebugReportsFields(test, uuid, debugReports);
}, `B&A forDebuggingOnly - missing required adTechOrigin`);

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly');
  let debugReports = [createDebugReportsPerOrigin(
      /*adTechOrigin=*/ 'http://nothttps.com',
      [createDebugReport(bidderDebugReportURL, /*isWinReport=*/ true)])];
  await testInvalidDebugReportsFields(test, uuid, debugReports);
}, `B&A forDebuggingOnly - HTTP adTechOrigin`);

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly');
  let debugReports = [createDebugReportsPerOrigin(
      /*adTechOrigin=*/ window.location.origin,
      [createDebugReport('http://nothttps.com', /*isWinReport=*/ true)])];
  await testInvalidDebugReportsFields(test, uuid, debugReports);
}, `B&A forDebuggingOnly - HTTP debug report url`);

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly');
  let debugReports = [createDebugReportsPerOrigin(
      /*adTechOrigin=*/ window.location.origin,
      [createDebugReport('not a url', /*isWinReport=*/ true)])];
  await testInvalidDebugReportsFields(test, uuid, debugReports);
}, `B&A forDebuggingOnly - debug report url not a url`);

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igOwner = OTHER_ORIGIN1;

  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly', igOwner);
  let sellerDebugReportURL =
      createSellerReportURL(uuid, /*id=*/ 'forDebuggingOnly');

  let debugReports = [
    createDebugReportsPerOrigin(
        igOwner,
        [createDebugReport(bidderDebugReportURL, /*isWinReport=*/ true)]),
    createDebugReportsPerOrigin(
        window.location.origin, [createDebugReport(
                                    sellerDebugReportURL, /*isWinReport=*/ true,
                                    /*isSellerReport=*/ true)])
  ];

  let result = await BA.testWithMutatedServerResponse(
      test, /*expectWin=*/ true,
      (msg) => {
        msg.debugReports = debugReports;
      },
      (ig, uuid) => {
        ig.ads[0].renderURL = createRenderURL(uuid);
      },
      igOwner);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(
      uuid, [bidderDebugReportURL, sellerDebugReportURL]);
}, `B&A forDebuggingOnly - debug reports sent`);

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  const igOwner = OTHER_ORIGIN1;

  let bidderDebugReportURL =
      createBidderReportURL(uuid, /*id=*/ 'forDebuggingOnly', igOwner);
  let sellerDebugReportURL =
      createSellerReportURL(uuid, /*id=*/ 'forDebuggingOnly');

  let debugReports = [
    createDebugReportsPerOrigin(
        igOwner, [createDebugReport('not a url', /*isWinReport=*/ true)]),
    createDebugReportsPerOrigin(
        window.location.origin, [createDebugReport(
                                    sellerDebugReportURL, /*isWinReport=*/ true,
                                    /*isSellerReport=*/ true)])
  ];

  let result = await BA.testWithMutatedServerResponse(
      test, /*expectWin=*/ true,
      (msg) => {
        msg.debugReports = debugReports;
      },
      (ig, uuid) => {
        ig.ads[0].renderURL = createRenderURL(uuid);
      },
      igOwner);
  createAndNavigateFencedFrame(test, result);
  await waitForObservedRequests(uuid, [sellerDebugReportURL]);
}, `B&A forDebuggingOnly - invalid debug reports don't affect other debug reports`);

// TODO(qingxinwu): multi seller auctions.
