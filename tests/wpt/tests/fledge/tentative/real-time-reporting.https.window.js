// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=third_party/cbor-js/cbor.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-last

'use strict;'

// The tests in this file focus on real time reporting.

// Creates a bidding script located on "origin". The generateBid() method calls
// real time reporting API and returns a bid of "bid".
// The reportWin() method is empty.
function createBiddingScriptURLForRealTimeReporting(origin = null, bid = 1) {
  return createBiddingScriptURL({
    origin: origin ? origin : new URL(BASE_URL).origin,
    bid: bid,
    allowComponentAuction: true,
    generateBid: `
      realTimeReporting.contributeToHistogram({ bucket: 20, priorityWeight: 1});`
  });
}

// Creates a decision script that calls real time reporting API.
// The reportResult() method is empty.
function createDecisionScriptURLForRealTimeReporting(uuid) {
  return createDecisionScriptURL(uuid, {
    scoreAd: `
      realTimeReporting.contributeToHistogram({ bucket: 200, priorityWeight: 1});`
  });
}

// Delay method that waits for prescribed number of milliseconds.
const delay = ms => new Promise(resolve => step_timeout(resolve, ms));

// Polls the given `origin` to retrieve reports sent there. Once the reports are
// received, returns the list of reports. Returns null if the timeout is reached
// before a report is available.
const pollReports = async (origin, wait_for = 1, timeout = 5000 /*ms*/) => {
  const path = '/.well-known/interest-group/real-time-report';
  let startTime = performance.now();
  let payloads = [];
  while (performance.now() - startTime < timeout) {
    const resp = await fetch(new URL(path, origin));
    const payload = await resp.arrayBuffer();
    if (payload.byteLength > 0) {
      payloads = payloads.concat(payload);
    }
    if (payloads.length >= wait_for) {
      return payloads;
    }
    await delay(/*ms=*/ 100);
  }
  if (payloads.length > 0) {
    return payloads;
  }
  return null;
};

// Verifies that `reports` has 1 report in cbor, which has the expected three
// fields.
// `version` should be 1.
// `histogram` and `platformHistogram` should be objects that pass
// verifyHistogram().
const verifyReports = (reports) => {
  assert_equals(reports.length, 1);
  const report = CBOR.decode(reports[0]);
  assert_own_property(report, 'version');
  assert_equals(report.version, 1);
  assert_own_property(report, 'histogram');
  verifyHistogram(report.histogram, 128, 1024);
  assert_own_property(report, 'platformHistogram');
  verifyHistogram(report.platformHistogram, 1, 4);
  assert_equals(Object.keys(report).length, 3);
};

// Verifies that a `histogram` has two fields: "buckets" and "length", where
// "buckets" field is a Uint8Array of `bucketSize`, and "length" field equals to
// `length`.
const verifyHistogram = (histogram, bucketSize, length) => {
  assert_own_property(histogram, 'buckets');
  assert_own_property(histogram, 'length');
  assert_equals(Object.keys(histogram).length, 2);
  assert_true(histogram.buckets instanceof Uint8Array);
  assert_equals(histogram.buckets.length, bucketSize);
  assert_equals(histogram.length, length);
};

const resetWptServer = () => Promise.all([
  resetReports('/.well-known/interest-group/real-time-report'),
]);

// Method to clear the stash. Takes the URL as parameter.
const resetReports = url => {
  // The view of the stash is path-specific
  // (https://web-platform-tests.org/tools/wptserve/docs/stash.html), therefore
  // the origin doesn't need to be specified.
  url = `${url}?clear_stash=true`;
  const options = {
    method: 'POST',
  };
  return fetch(url, options);
};

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1],
    sellerRealTimeReportingConfig: {type: 'default-local-reporting'},
    perBuyerRealTimeReportingConfig:
        {[OTHER_ORIGIN1]: {type: 'default-local-reporting'}}
  });
  const sellerReports = await pollReports(location.origin);
  verifyReports(sellerReports);

  const buyerReports = await pollReports(OTHER_ORIGIN1);
  verifyReports(buyerReports);
}, 'Real time reporting different buyer and seller both opted-in and called api.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1],
    perBuyerRealTimeReportingConfig:
        {[OTHER_ORIGIN1]: {type: 'default-local-reporting'}}
  });

  const buyerReports = await pollReports(OTHER_ORIGIN1);
  verifyReports(buyerReports);

  // Seller called the RTR API, but didn't opt-in.
  const sellerReports =
      await pollReports(location.origin, /*wait_for=*/ 1, /*timeout=*/ 1000);
  assert_equals(sellerReports, null);
}, 'Real time reporting buyer opted-in but not seller.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1],
    sellerRealTimeReportingConfig: {type: 'default-local-reporting'}
  });

  const sellerReports = await pollReports(location.origin);
  verifyReports(sellerReports);

  // Buyer called the RTR API, but didn't opt-in.
  const buyerReports =
      await pollReports(OTHER_ORIGIN1, /*wait_for=*/ 1, /*timeout=*/ 1000);
  assert_equals(buyerReports, null);
}, 'Real time reporting seller opted-in but not buyer.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(
      test, uuid, OTHER_ORIGIN1,
      {biddingLogicURL: createBiddingScriptURL({origin: OTHER_ORIGIN1})});
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURL(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1],
    sellerRealTimeReportingConfig: {type: 'default-local-reporting'},
    perBuyerRealTimeReportingConfig:
        {[OTHER_ORIGIN1]: {type: 'default-local-reporting'}}
  });
  const sellerReports = await pollReports(location.origin);
  verifyReports(sellerReports);

  const buyerReports = await pollReports(OTHER_ORIGIN1);
  verifyReports(buyerReports);
}, 'Real time reporting different buyer and seller both opted-in but did not call api.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1]
  });
  const sellerReports = await pollReports(location.origin);
  assert_equals(sellerReports, null);
  const buyerReports =
      await pollReports(OTHER_ORIGIN1, /*wait_for=*/ 1, /*timeout=*/ 1000);
  assert_equals(buyerReports, null);
}, 'Real time reporting both called api but did not opt-in.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinInterestGroup(
      test, uuid,
      {biddingLogicURL: createBiddingScriptURLForRealTimeReporting()});

  const origin = location.origin;
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    sellerRealTimeReportingConfig: {type: 'default-local-reporting'},
    perBuyerRealTimeReportingConfig:
        {[origin]: {type: 'default-local-reporting'}}
  });
  const reports = await pollReports(origin);
  verifyReports(reports);
}, 'Real time reporting buyer and seller same origin.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN2, {
    biddingLogicURL:
        createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN2, /*bid=*/ 100)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1, OTHER_ORIGIN2],
    perBuyerRealTimeReportingConfig: {
      [OTHER_ORIGIN1]: {type: 'default-local-reporting'},
      [OTHER_ORIGIN2]: {type: 'default-local-reporting'}
    }
  });
  const reports1 = await pollReports(OTHER_ORIGIN1);
  verifyReports(reports1);

  const reports2 = await pollReports(OTHER_ORIGIN2);
  verifyReports(reports2);
}, 'Real time reporting both winning and losing buyers opted-in.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await resetWptServer();
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1, {
    biddingLogicURL: createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN1)
  });
  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN2, {
    biddingLogicURL:
        createBiddingScriptURLForRealTimeReporting(OTHER_ORIGIN2, /*bid=*/ 100)
  });
  await runBasicFledgeAuctionAndNavigate(test, uuid, {
    decisionLogicURL: createDecisionScriptURLForRealTimeReporting(uuid),
    interestGroupBuyers: [OTHER_ORIGIN1, OTHER_ORIGIN2],
    perBuyerRealTimeReportingConfig:
        {[OTHER_ORIGIN1]: {type: 'default-local-reporting'}}
  });
  const reports1 = await pollReports(OTHER_ORIGIN1);
  verifyReports(reports1);

  const reports2 =
      await pollReports(OTHER_ORIGIN2, /*wait_for=*/ 1, /*timeout=*/ 1000);
  assert_equals(reports2, null);
}, 'Real time reporting one buyer opted-in but not the other.');
